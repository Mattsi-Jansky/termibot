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
        Ok(())
    }
}

impl Default for DefaultActionHandler{
    fn default() -> Self {
        Self {}
    }
}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn given_channel_message_action_should_send_message_to_channel() {
//         let handler = DefaultActionHandler::default();
//         let mock_
//         let test_action = Action::MessageChannel { channel: String::from("#bots"), message: String::from("hello world")};
//
//         handler.handle(test_action, )
//     }
// }