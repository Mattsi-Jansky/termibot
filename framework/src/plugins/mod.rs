use crate::actions::Action;
use async_trait::async_trait;
use client::models::socket_message::Event;
use mockall::automock;
use crate::dependencies::Dependencies;

#[async_trait]
#[automock]
pub trait Plugin {
    async fn on_event(&self, event: &Event, dependencies: &Dependencies) -> Action;
}
