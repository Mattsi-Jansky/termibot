use async_trait::async_trait;
use mockall::automock;
use client::models::socket_message::Event;
use crate::actions::Action;

#[async_trait]
#[automock]
pub trait Plugin {
    async fn on_event(&self, event: &Event) -> Action;
}
