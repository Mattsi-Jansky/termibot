use crate::core::client::SlackBotClient;
use slack_morphism::prelude::*;

pub async fn reply_to_thread(
    client: &Box<dyn SlackBotClient + Send + Sync>,
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
    use mockall::mock;

    #[tokio::test]
    async fn wow() {
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

        let incoming_message_event = SlackMessageEvent {
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
        let mut outgoing_message = MockMessageTemplate::new();
        outgoing_message
            .expect_render_template()
            .times(1)
            .returning(|| SlackMessageContent::new());
        let outgoing_message = Box::new(outgoing_message);
        let mut client = MockClient::new();
        client.expect_post_message().times(1).returning(|_| {
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
        let client = Box::new(client) as Box<dyn SlackBotClient + Send + Sync>;

        let result = reply_to_thread(&client, incoming_message_event, outgoing_message).await;
        assert!(matches!(result, Ok(_)));
    }
}
