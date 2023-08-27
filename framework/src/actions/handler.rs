use crate::actions::Action;
use async_trait::async_trait;
use client::error::SlackClientError;
use client::SlackClient;
use mockall::automock;
use std::sync::Arc;

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
            Action::MessageChannel { channel, message } => client
                .message_channel(&channel[..], &message)
                .await
                .map(|_| ())?,
            Action::ReplyToThread {
                channel,
                thread_id,
                message,
            } => client
                .message_thread(&channel, &thread_id, &message)
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
    use client::models::message_body::MessageBody;
    use client::models::message_id::MessageId;
    use client::MockSlackClient;

    #[tokio::test]
    async fn given_channel_message_action_should_send_message_to_channel() {
        let handler = DefaultActionHandler::default();
        let test_action = Action::MessageChannel {
            channel: String::from("#bots"),
            message: MessageBody::from_text("hello world"),
        };
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_message_channel()
            .withf(|channel, message| channel == "#bots" && message.get_text() == "hello world")
            .times(1)
            .returning(|_, _| {
                Ok(HttpApiResponse {
                    ok: true,
                    message: Some(Message {
                        id: "".to_string().into(),
                        text: "".to_string(),
                        user: "".to_string(),
                    }),
                    error: None,
                    errors: None,
                })
            });

        handler
            .handle(test_action, Arc::new(mock_client))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn given_thread_reply_should_send_message_to_thread() {
        let handler = DefaultActionHandler::default();
        let test_action = Action::ReplyToThread {
            channel: String::from("#bots"),
            thread_id: "thread-id".to_string().into(),
            message: MessageBody::from_text("hello world"),
        };
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_message_thread()
            .withf(|channel, thread, message| {
                channel == "#bots"
                    && thread == &MessageId::new(String::from("thread-id"))
                    && message.get_text() == "hello world"
            })
            .times(1)
            .returning(|_, _, _| {
                Ok(HttpApiResponse {
                    ok: true,
                    message: Some(Message {
                        id: "".to_string().into(),
                        text: "".to_string(),
                        user: "".to_string(),
                    }),
                    error: None,
                    errors: None,
                })
            });

        handler
            .handle(test_action, Arc::new(mock_client))
            .await
            .unwrap();
    }
}
