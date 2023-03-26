use slack_morphism::prelude::*;

use crate::actions::Action;
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
    ) -> Action;
}
