use async_trait::async_trait;
use crate::actions::Action;
use crate::actions::handlers::reply_thread::reply_to_thread;
use crate::core::client::SlackBotClient;

pub(crate) mod reply_thread;

#[async_trait]
pub trait ActionHandler {
    async fn resolve_action(
        &self,
        action: Action,
        client: &(dyn SlackBotClient + Send + Sync),
        errors: &mut Vec<Box<dyn std::error::Error + Send + Sync>>,
    );
}

pub struct DefaultActionHandler {}

impl DefaultActionHandler {
    pub fn new() -> DefaultActionHandler { DefaultActionHandler {} }
}

#[async_trait]
impl ActionHandler for DefaultActionHandler {
    async fn resolve_action(
        &self,
        action: Action,
        client: &(dyn SlackBotClient + Send + Sync),
        errors: &mut Vec<Box<dyn std::error::Error + Send + Sync>>,
    ) {
        match action {
            Action::DoNothing => {}
            Action::ReplyToThread(incoming_message_event, outgoing_message) => {
                let result = reply_to_thread(client, incoming_message_event, outgoing_message).await;
                if let Err(error) = result {
                    errors.push(error);
                }
            }
            Action::Error(error) => errors.push(error),
        }
    }
}
