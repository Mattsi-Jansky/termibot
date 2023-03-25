use slack_morphism::prelude::*;

use std::sync::Arc;
use async_trait::async_trait;

pub mod songlink;

#[async_trait]
pub trait Plugin {
    async fn push_event(
        &self,
        event: SlackPushEventCallback,
        client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
