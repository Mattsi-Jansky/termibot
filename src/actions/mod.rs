pub(crate) mod handlers;

use slack_morphism::prelude::{SlackMessageEvent, SlackMessageTemplate};

pub enum Action {
    DoNothing,
    ReplyToThread(SlackMessageEvent, Box<dyn SlackMessageTemplate + Send + Sync>),
    Error(Box<dyn std::error::Error + Send + Sync>),
}
