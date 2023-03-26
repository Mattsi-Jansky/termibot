use crate::core::client::SlackBotClient;
use slack_morphism::prelude::{SlackMessageEvent, SlackMessageTemplate};

pub(crate) mod handlers;

pub enum Action {
    DoNothing,
    ReplyToThread(
        SlackMessageEvent,
        Box<dyn SlackMessageTemplate + Send + Sync>,
    ),
    Error(Box<dyn std::error::Error + Send + Sync>),
}
