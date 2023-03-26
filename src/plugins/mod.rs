use slack_morphism::prelude::*;

use async_trait::async_trait;
use std::sync::Arc;

pub mod songlink;

#[async_trait]
pub trait Plugin {
    fn new() -> Self
    where
        Self: Sized;

    async fn push_event(
        &self,
        event: SlackPushEventCallback,
        client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> EventResponse;
}

pub enum EventResponse {
    DoNothing,
    ReplyToThread(SlackMessageEvent, Box<dyn SlackMessageTemplate + Send + Sync>),
    Error(Box<dyn std::error::Error + Send + Sync>),
}
