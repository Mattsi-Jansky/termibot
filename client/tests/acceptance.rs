mod test_client_builder;

use crate::test_client_builder::TestClientBuilder;
use client::models::blocks::card::CardBlock;
use client::models::blocks::context::{ContextBlock, ContextElement};
use client::models::blocks::elements::button::{ButtonElement, ButtonStyle};
use client::models::blocks::elements::image::ImageElement;
use client::models::blocks::image::ImageBlock;
use client::models::blocks::objects::text::{Text, TextBody};
use client::models::blocks::section::SectionBlock;
use client::models::blocks::Block;
use client::models::file_upload::FileUpload;
use client::models::message_body::MessageBody;
use std::time::SystemTime;

const ORANGE_SQUARE_PNG: &[u8] = include_bytes!("fixtures/orange_square.png");
const PUBLIC_IMAGE_URL: &str = "https://files.mastodon.social/media_attachments/files/116/929/144/353/146/667/original/4541caf1a6f64af8.png";

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
async fn should_upload_an_image_and_show_it_in_a_thread_reply() {
    let builder = TestClientBuilder::new("should_upload_an_image_and_show_it_in_a_thread_reply");
    let client = builder.new_client();

    let file_id = client
        .upload_file(
            FileUpload::new("orange_square.png", ORANGE_SQUARE_PNG.to_vec())
                .with_alt_text("An orange square"),
        )
        .await
        .expect("upload should succeed");
    let parent = client
        .message_channel("#bots", &MessageBody::from_text("Testing image uploads"))
        .await
        .expect("parent message should send")
        .message
        .unwrap();
    let reply = MessageBody::new(
        vec![
            SectionBlock::new_markdown("This image was uploaded by the bot:"),
            ImageBlock::from_slack_file(&file_id, "An orange square").into(),
        ],
        Some("Uploaded image".to_string()),
    )
    .unwrap();
    let result = client.message_thread("#bots", &parent.id, &reply).await;

    assert!(result.is_ok(), "{result:?}");
}

#[tokio::test]
async fn should_send_context_and_linked_image_blocks_without_unfurling() {
    let builder =
        TestClientBuilder::new("should_send_context_and_linked_image_blocks_without_unfurling");
    let client = builder.new_client();
    let message = MessageBody::new(
        vec![
            ContextBlock::new(vec![
                ContextElement::image(PUBLIC_IMAGE_URL, "Mastodon's profile page"),
                ContextElement::markdown("*Termibot* <https://joinmastodon.org|joinmastodon.org>"),
            ])
            .into(),
            SectionBlock::new_markdown(
                "Unfurling is off, so this link shouldn't get a preview: https://joinmastodon.org",
            ),
            ImageBlock::from_url(PUBLIC_IMAGE_URL, "Mastodon's profile page")
                .with_title("Linked image")
                .into(),
        ],
        Some("Linked image".to_string()),
    )
    .unwrap()
    .without_unfurling();

    let result = client.message_channel("#bots", &message).await;

    assert!(result.is_ok(), "{result:?}");
}

#[tokio::test]
async fn should_send_a_card_with_an_image_and_a_link_button() {
    let builder = TestClientBuilder::new("should_send_a_card_with_an_image_and_a_link_button");
    let client = builder.new_client();
    let card = CardBlock::default()
        .with_hero_image(ImageElement::from_url(
            PUBLIC_IMAGE_URL,
            "Mastodon's profile page",
        ))
        .with_title(Text::plain("Video"))
        .with_subtitle(Text::plain("1:21"))
        .with_body(Text::plain(
            "A card like the ones Mastodon previews use for videos",
        ))
        .with_actions(vec![ButtonElement::link(
            "▶ Watch",
            "https://joinmastodon.org",
        )
        .with_style(ButtonStyle::Primary)]);
    let message = MessageBody::new(vec![card.into()], Some("Card".to_string())).unwrap();

    let result = client.message_channel("#bots", &message).await;

    assert!(result.is_ok(), "{result:?}");
}

#[tokio::test]
async fn should_refuse_messages_with_images_slack_cannot_fetch() {
    let builder = TestClientBuilder::new("should_refuse_messages_with_images_slack_cannot_fetch");
    let client = builder.new_client();
    let message = MessageBody::new(
        vec![ImageBlock::from_url(
            "https://joinmastodon.org/this-image-does-not-exist.png",
            "Missing image",
        )
        .into()],
        Some("Missing image".to_string()),
    )
    .unwrap();

    let result = client.message_channel("#bots", &message).await;

    assert!(result.unwrap_err().0.contains("invalid_blocks"));
}

#[tokio::test]
async fn should_get_identity() {
    let builder = TestClientBuilder::new("should_get_identity");
    let client = builder.new_client();

    let response = client.get_identity().await;

    assert!(response.unwrap().ok);
}
