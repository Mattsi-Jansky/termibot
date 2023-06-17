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
