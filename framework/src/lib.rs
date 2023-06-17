extern crate core;

use async_trait::async_trait;
use futures::future::join_all;
use futures::StreamExt;
use client::error::SlackClientError;
use client::models::response::Message;
use client::models::{Event, SocketMessage};
use client::socket_listener::SocketModeListener;
use client::SlackClient;
use plugins::Plugin;
use tracing::info;
use crate::actions::handler::ActionHandler;

pub mod plugins;
pub mod actions;

pub struct SlackBot {
    client: Box<dyn SlackClient + Send + Sync>,
    listener: Box<dyn SocketModeListener>,
    plugins: Vec<Box<dyn Plugin>>,
    action_handler: Box<dyn ActionHandler>
}

impl SlackBot {
    pub fn from(client: Box<dyn SlackClient + Send + Sync>, listener: Box<dyn SocketModeListener>, handler: Box<dyn ActionHandler>) -> SlackBot {
        SlackBot { client, listener, plugins: vec![], action_handler: handler }
    }

    pub async fn run(self) -> Result<(), SlackClientError> {
        let mut listener = self.client.connect_to_socket_mode().await?;
        info!("Slack bot starting");

        loop {
            let message = listener.next().await?;
            let mut actions = vec![];
            info!("Received message: {message:?}");

            match &message {
                SocketMessage::Event {
                    envelope_id: _,
                    payload,
                } => {
                    for plugin in &self.plugins {
                        let action = plugin.on_event(&payload.event);
                        actions.push(action);
                    }
                }
                SocketMessage::Interactive { .. } => {
                    info!("Cannot handle interactive events yet, not implemented.")
                }
                SocketMessage::SlashCommand { .. } => {
                    info!("Cannot handle slash commands yet, not implemented.")
                }

                SocketMessage::Hello { .. } => { /* Nothing to do */ }
                SocketMessage::Disconnect { .. } => {
                    break;
                }
            }

            join_all(
                join_all(actions).await
                    .into_iter()
                    .map(|action| self.action_handler.handle(action, &self.client))
            ).await;
        }

        info!("Slack bot finishing");
        Ok(())
    }

    pub(crate) fn with(mut self, plugin: Box<dyn Plugin>) -> Self {
        self.plugins.push(plugin);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::future;
    use super::*;
    use async_trait::async_trait;
    use client::error::SlackClientError;
    use client::models::{Payload, SocketMessage};
    use client::models::response::ApiResponse;
    use crate::actions::Action;
    use plugins::MockPlugin;
    use actions::handler::MockActionHandler;
    use client::MockSlackClient;

    struct TestSocketModeListener {
        call_count: usize,
    }
    impl Default for TestSocketModeListener {
        fn default() -> Self {
            TestSocketModeListener { call_count: 0 }
        }
    }
    #[async_trait]
    impl SocketModeListener for TestSocketModeListener {
        async fn next(&mut self) -> serde_json::error::Result<SocketMessage> {
            self.call_count += 1;

            if self.call_count == 1 {
                Ok(SocketMessage::Hello {})
            } else if self.call_count == 2 {
                Ok(SocketMessage::Event {
                    envelope_id: "fake-envelope-id".to_string(),
                    payload: Payload {
                        event: Event {
                            id: "fake-id".to_string(),
                            event_type: "fake-id".to_string(),
                            text: Some("this is your secret message".to_string()),
                            user: None,
                            blocks: vec![],
                            channel: None,
                            channel_type: None,
                        },
                    },
                })
            } else {
                Ok(SocketMessage::Disconnect {})
            }
        }
    }

    #[tokio::test]
    async fn disconnect_after_disconnect_message_received() {
        let bot = SlackBot::from(
            mock_client(),
            Box::new(TestSocketModeListener::default()),
            Box::new(MockActionHandler::new())
        );

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn forward_event_message_to_plugin() {
        let mut mock_action_handler = Box::new(MockActionHandler::new());
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin.expect_on_event()
            .times(1)
            .returning(|_| Box::pin(future::ready(Action::DoNothing)));
        mock_action_handler.expect_handle().times(1).returning(|_, _| Box::pin(future::ready(Ok(()))));
        let bot = SlackBot::from(
            mock_client(),
            Box::new(TestSocketModeListener::default()),
            mock_action_handler
        )
            .with(mock_plugin);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn forward_event_outcome_to_action_handler() {
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin.expect_on_event()
            .returning(|_| Box::pin(future::ready(Action::MessageChannel
            {
                channel: String::from("my test channel"),
                message: String::from("my test message")
            })));
        let mut mock_action_handler = Box::new(MockActionHandler::new());
        mock_action_handler.expect_handle().times(1)
            .withf(|action, client| match action {
                Action::MessageChannel { channel, message } =>
                    channel == "my test channel" && message == "my test message",
                _ => false
            })
            .returning(|_, _| Box::pin(future::ready(Ok(()))));
        let bot = SlackBot::from(
            mock_client(),
            Box::new(TestSocketModeListener::default()),
            mock_action_handler
        )
            .with(mock_plugin);

        bot.run().await.unwrap();
    }

    fn mock_client() -> Box<MockSlackClient> {
        let mut mock_slack_client = Box::new(MockSlackClient::new());
        mock_slack_client.expect_connect_to_socket_mode()
            .times(1)
            .returning(|| Ok(Box::new(TestSocketModeListener::default())));
        mock_slack_client
    }
}
