use async_trait::async_trait;
use mockall::automock;
use client::error::SlackClientError;
use client::SlackClient;
use crate::actions::Action;

#[async_trait]
#[automock]
pub trait ActionHandler {
    async fn handle(&self, action: Action, client: &Box<dyn SlackClient + Send + Sync>) -> Result<(), SlackClientError>;
}

pub struct DefaultActionHandler {}

#[async_trait]
impl ActionHandler for DefaultActionHandler {
    async fn handle(&self, action: Action, client: &Box<dyn SlackClient + Send + Sync>) -> Result<(), SlackClientError> {
        match action {
            Action::DoNothing => {}
            Action::MessageChannel { channel, message } => {
                client.message_channel(&channel[..], &message[..]).await
                    .map(|_| ())?
            },
            Action::ReplyToThread { channel, thread, message } => {
                client.message_thread(&channel, &thread, &message).await
                    .map(|_| ())?
            }
        }

        Ok(())
    }
}

impl Default for DefaultActionHandler{
    fn default() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use client::message::Message;
    use client::MockSlackClient;
    use client::response::ApiResponse;
    use super::*;

    #[tokio::test]
    async fn given_channel_message_action_should_send_message_to_channel() {
        let handler = DefaultActionHandler::default();
        let test_action = Action::MessageChannel {
            channel: String::from("#bots"),
            message: String::from("hello world")
        };
        let mut mock_client = Box::new(MockSlackClient::new());
        mock_client.expect_message_channel()
            .withf(|channel, message|channel == "#bots" && message ==  "hello world")
            .times(1)
            .returning(|_,_|Ok(ApiResponse{ ok: true, message: Message {
                id: "".to_string(),
                text: "".to_string(),
                user: "".to_string(),
            } }));

        handler.handle(test_action, &(mock_client as Box<dyn SlackClient + Send + Sync>)).await.unwrap();
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
            message: String::from("hello world")
        };
        let mut mock_client = Box::new(MockSlackClient::new());
        mock_client.expect_message_thread()
            .withf(|channel, thread, message| channel == "#bots"
                && thread == &Message {
                id: "thread-id".to_string(),
                text: "parent-message".to_string(),
                user: "parent-user".to_string(),
            }
                && message == "hello world")
            .times(1)
            .returning(|_,_,_|Ok(ApiResponse{ ok: true, message: Message {
                id: "".to_string(),
                text: "".to_string(),
                user: "".to_string(),
            } }));

        handler.handle(test_action, &(mock_client as Box<dyn SlackClient + Send + Sync>)).await.unwrap();
    }
}
