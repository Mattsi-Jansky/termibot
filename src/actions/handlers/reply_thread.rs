use crate::core::client::SlackBotClient;
use slack_morphism::prelude::*;

pub async fn reply_to_thread(
    client: &(dyn SlackBotClient + Send + Sync),
    message_to_reply_to: SlackMessageEvent,
    outgoing_message: Box<dyn SlackMessageTemplate + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ts = message_to_reply_to.origin.ts.clone();
    let channel = message_to_reply_to.origin.channel.clone().unwrap();

    let request = SlackApiChatPostMessageRequest {
        channel,
        content: outgoing_message.render_template(),
        as_user: None,
        icon_emoji: None,
        icon_url: None,
        link_names: None,
        parse: None,
        thread_ts: Some(ts),
        username: None,
        reply_broadcast: None,
        unfurl_links: None,
        unfurl_media: None,
    };

    client.post_message(&request).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use lazy_static::lazy_static;
    use mockall::{mock, predicate};

    mock! {
        #[async_trait]
        Client {}
        #[async_trait]
        impl SlackBotClient for Client {
            async fn post_message(&self, request: &SlackApiChatPostMessageRequest) -> ClientResult<SlackApiChatPostMessageResponse>;
        }
    }

    mock! {
        MessageTemplate {}     // Name of the mock struct, less the "Mock" prefix
        impl SlackMessageTemplate for MessageTemplate {   // specification of the trait to mock
            fn render_template(&self) -> SlackMessageContent;
        }
    }

    lazy_static! {
        static ref TEST_MESSAGE_EVENT: SlackMessageEvent = SlackMessageEvent {
            origin: SlackMessageOrigin {
                ts: SlackTs::from("123"),
                channel: Some(SlackChannelId::from("S0M3L0NG1D")),
                channel_type: None,
                thread_ts: None,
                client_msg_id: None,
            },
            content: None,
            sender: SlackMessageSender {
                user: None,
                bot_id: None,
                username: None,
                display_as_bot: None,
            },
            subtype: None,
            hidden: None,
            edited: None,
            deleted_ts: None,
        };
    }

    fn test_message_content() -> SlackMessageContent {
        SlackMessageContent::new()
            .with_text(String::from("my test message"))
    }

    #[tokio::test]
    async fn should_extract_origin_details_and_send_message_replying_to_thread() {
        let mut outgoing_message = MockMessageTemplate::new();
        outgoing_message
            .expect_render_template()
            .times(1)
            .returning(|| test_message_content());
        let outgoing_message = Box::new(outgoing_message);
        let mut client = MockClient::new();
        client.expect_post_message()
            .with(predicate::function(|request: &SlackApiChatPostMessageRequest|
                request.thread_ts.as_ref().unwrap().0 == "123"
                && request.channel.0 == "S0M3L0NG1D"
                && request.content.eq(&test_message_content())
            ))
            .times(1)
            .returning(|_| {
            Ok(SlackApiChatPostMessageResponse {
                channel: SlackChannelId::from("S0M3L0NG1D"),
                ts: SlackTs::from("123"),
                message: SlackMessage {
                    origin: SlackMessageOrigin::new(SlackTs::from("123")),
                    content: SlackMessageContent::new(),
                    sender: SlackMessageSender::new(),
                    parent: SlackParentMessageParams::new(),
                },
            })
        });

        let result = reply_to_thread(&client, TEST_MESSAGE_EVENT.clone(), outgoing_message).await;

        assert!(matches!(result, Ok(_)));
    }
}
