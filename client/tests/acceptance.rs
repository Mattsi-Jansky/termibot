mod test_client_builder;

use crate::test_client_builder::TestClientBuilder;
use client::models::blocks::objects::text::{Text, TextBody};
use client::models::blocks::section::SectionBlock;
use client::models::blocks::Block;
use client::models::message_body::MessageBody;
use std::time::SystemTime;

use client::SlackClient;

#[tokio::test]
async fn should_send_messages_to_channels_and_threads() {
    let builder = TestClientBuilder::new("should_send_messages_to_channels_and_threads");
    let client = builder.new_client();

    let result = client
        .message_channel("#bots", &MessageBody::from_text("foobar"))
        .await;
    assert!(result.is_ok());
    let message = result.unwrap().message.unwrap();

    let new_message = MessageBody::from_text(&format!("replying to {}", message.text)[..]);
    let result2 = client
        .message_thread("#bots", &message.id, &new_message)
        .await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn given_too_many_requests_should_throttle_to_avoid_rate_limit() {
    let builder =
        TestClientBuilder::new("given_too_many_requests_should_throttle_to_avoid_rate_limit");
    let client = builder.new_client();

    let before = SystemTime::now();
    for _i in 0..9 {
        client
            .message_channel("#bots", &MessageBody::from_text("foobar"))
            .await
            .expect("Should succeed");
    }

    let duration = before.elapsed().unwrap().as_millis();
    assert!(duration >= 75, "Wrong duration: {}", duration);
}

#[tokio::test]
async fn given_blocks_api_used_should_send_blocks_message() {
    let builder = TestClientBuilder::new("given_blocks_api_used_should_send_blocks_message");
    let client = builder.new_client();

    let blocks = vec![
        Block::Section(
            SectionBlock::new()
                .text(Some(Text::Markdown(
                    TextBody::new()
                        .text("*why* _hello_ there".to_string())
                        .build(),
                )))
                .build(),
        ),
        Block::Divider,
        Block::Section(
            SectionBlock::new()
                .fields(Some(vec![
                    Text::PlainText(TextBody::new().text("foo".to_string()).build()),
                    Text::PlainText(TextBody::new().text("bar".to_string()).build()),
                ]))
                .build(),
        ),
    ];
    let result = client
        .message_channel("#bots", &MessageBody::new(blocks, None).unwrap())
        .await;

    assert!(result.unwrap().ok);
}

#[tokio::test]
async fn should_get_bot_user_details() {
    let builder = TestClientBuilder::new("should_get_bot_user_details");
    let client = builder.new_client();

    let id = client.get_user_id().await.unwrap();

    assert_eq!(id, String::from("UE02Q1FTK"))
}
