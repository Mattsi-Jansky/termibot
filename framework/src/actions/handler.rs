use crate::actions::Action;
use async_trait::async_trait;
use client::error::SlackClientError;
use client::SlackClient;
use mockall::automock;
use std::sync::Arc;
use tracing::warn;

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
            Action::FirstSuccessful(actions) => {
                self.handle_first_successful(actions, client).await?
            }
        }

        Ok(())
    }
}

impl DefaultActionHandler {
    async fn handle_first_successful(
        &self,
        actions: Vec<Action>,
        client: Arc<dyn SlackClient + Send + Sync>,
    ) -> Result<(), SlackClientError> {
        let mut last_error = SlackClientError("There were no actions to try".to_string());
        for action in actions {
            match self.handle(action, client.clone()).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    warn!("Action failed, trying the next one: {:?}", err);
                    last_error = err;
                }
            }
        }

        Err(last_error)
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

    #[tokio::test]
    async fn first_successful_stops_at_the_first_action_that_succeeds() {
        let mut mock_client = MockSlackClient::new();
        expect_channel_message(&mut mock_client, "first", 1, || Ok(sent()));
        expect_channel_message(&mut mock_client, "second", 0, || Ok(sent()));

        let result = DefaultActionHandler::default()
            .handle(
                Action::FirstSuccessful(vec![channel_message("first"), channel_message("second")]),
                Arc::new(mock_client),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn first_successful_tries_the_next_action_when_one_fails() {
        let mut mock_client = MockSlackClient::new();
        expect_channel_message(&mut mock_client, "first", 1, || {
            Err(failure("first failed"))
        });
        expect_channel_message(&mut mock_client, "second", 1, || Ok(sent()));

        let result = DefaultActionHandler::default()
            .handle(
                Action::FirstSuccessful(vec![channel_message("first"), channel_message("second")]),
                Arc::new(mock_client),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn first_successful_fails_with_the_last_error_when_every_action_fails() {
        let mut mock_client = MockSlackClient::new();
        expect_channel_message(&mut mock_client, "first", 1, || {
            Err(failure("first failed"))
        });
        expect_channel_message(&mut mock_client, "second", 1, || {
            Err(failure("second failed"))
        });

        let result = DefaultActionHandler::default()
            .handle(
                Action::FirstSuccessful(vec![channel_message("first"), channel_message("second")]),
                Arc::new(mock_client),
            )
            .await;

        assert_eq!(result.unwrap_err().0, "second failed");
    }

    fn channel_message(text: &str) -> Action {
        Action::MessageChannel {
            channel: String::from("#bots"),
            message: MessageBody::from_text(text),
        }
    }

    fn expect_channel_message(
        mock_client: &mut MockSlackClient,
        text: &'static str,
        times: usize,
        result: fn() -> Result<HttpApiResponse, SlackClientError>,
    ) {
        mock_client
            .expect_message_channel()
            .withf(move |_, message| message.get_text() == text)
            .times(times)
            .returning(move |_, _| result());
    }

    fn sent() -> HttpApiResponse {
        HttpApiResponse {
            ok: true,
            message: None,
            error: None,
            errors: None,
        }
    }

    fn failure(error: &str) -> SlackClientError {
        SlackClientError(error.to_string())
    }
}
