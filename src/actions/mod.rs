use crate::actions::handlers::reply_thread::reply_to_thread;
use slack_morphism::prelude::{SlackHyperClient, SlackMessageEvent, SlackMessageTemplate};
use std::sync::Arc;

pub(crate) mod handlers;

pub enum Action {
    DoNothing,
    ReplyToThread(
        SlackMessageEvent,
        Box<dyn SlackMessageTemplate + Send + Sync>,
    ),
    Error(Box<dyn std::error::Error + Send + Sync>),
}

pub async fn resolve_action(
    action: Action,
    client: &Arc<SlackHyperClient>,
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
