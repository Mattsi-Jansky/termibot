use std::sync::Arc;
use crate::actions::Action;
use async_trait::async_trait;
use client::error::SlackClientError;
use client::SlackClient;
use mockall::automock;

#[async_trait]
#[automock]
pub trait ActionHandler {
    async fn handle(
        &self,
        action: Action,
        client: Arc<dyn SlackClient + Send + Sync>,
    ) -> Result<(), SlackClientError>;
}

#[derive(Default)]
pub struct DefaultActionHandler {}

#[async_trait]
impl ActionHandler for DefaultActionHandler {
    async fn handle(
        &self,
        action: Action,
        client: Arc<dyn SlackClient + Send + Sync>,
    ) -> Result<(), SlackClientError> {
        match action {
            Action::DoNothing => {}
            Action::MessageChannel { channel, message } => client
                .message_channel(&channel[..], &message[..])
                .await
                .map(|_| ())?,
            Action::ReplyToThread {
                channel,
                thread,
                message,
            } => client
                .message_thread(&channel, &thread, &message)
                .await
                .map(|_| ())?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::models::http_response::HttpApiResponse;
    use client::models::http_response::Message;
    use client::MockSlackClient;

    #[tokio::test]
    async fn given_channel_message_action_should_send_message_to_channel() {
        let handler = DefaultActionHandler::default();
        let test_action = Action::MessageChannel {
            channel: String::from("#bots"),
            message: String::from("hello world"),
        };
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_message_channel()
            .withf(|channel, message| channel == "#bots" && message == "hello world")
            .times(1)
            .returning(|_, _| {
                Ok(HttpApiResponse {
                    ok: true,
                    message: Message {
                        id: "".to_string(),
                        text: "".to_string(),
                        user: "".to_string(),
                    },
                })
            });

        handler
            .handle(
                test_action,
                Arc::new(mock_client),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn given_thread_reply_should_send_message_to_thread() {
        let handler = DefaultActionHandler::default();
        let test_action = Action::ReplyToThread {
            channel: String::from("#bots"),
            thread: Message {
                id: "thread-id".to_string(),
                text: "parent-message".to_string(),
                user: "parent-user".to_string(),
            },
            message: String::from("hello world"),
        };
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_message_thread()
            .withf(|channel, thread, message| {
                channel == "#bots"
                    && thread
                        == &Message {
                            id: "thread-id".to_string(),
                            text: "parent-message".to_string(),
                            user: "parent-user".to_string(),
                        }
                    && message == "hello world"
            })
            .times(1)
            .returning(|_, _, _| {
                Ok(HttpApiResponse {
                    ok: true,
                    message: Message {
                        id: "".to_string(),
                        text: "".to_string(),
                        user: "".to_string(),
                    },
                })
            });

        handler
            .handle(
                test_action,
                Arc::new(mock_client),
            )
            .await
            .unwrap();
    }
}
