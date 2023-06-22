extern crate core;

use crate::actions::handler::{ActionHandler, DefaultActionHandler};
use client::error::SlackClientError;
use client::models::socket_message::SocketMessage;
use client::{ReqwestSlackClient, SlackClient};
use futures::future::join_all;
use plugins::Plugin;
use std::sync::Arc;
use futures::TryFutureExt;
use tracing::{error, info};

pub mod actions;
pub mod plugins;

pub struct SlackBot {
    client: Arc<dyn SlackClient + Send + Sync>,
    plugins: Vec<Box<dyn Plugin>>,
    action_handler: Box<dyn ActionHandler>,
}

impl SlackBot {
    pub fn new(bot_token: &str, app_token: &str) -> Self {
        Self {
            client: Arc::new(ReqwestSlackClient::new(bot_token, app_token)),
            plugins: vec![],
            action_handler: Box::new(DefaultActionHandler{}),
        }
    }

    pub fn from(
        client: Arc<dyn SlackClient + Send + Sync>,
        handler: Box<dyn ActionHandler>,
    ) -> SlackBot {
        SlackBot {
            client,
            plugins: vec![],
            action_handler: handler,
        }
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

            let results = join_all(
                join_all(actions)
                    .await
                    .into_iter()
                    .map(|action| self.action_handler.handle(action, self.client.clone())),
            ).await;

            for result in results {
                if let Err(err) = result {
                    error!("Error occurred when trying to execute action: {:?}", err);
                }
            }
        }

        info!("Slack bot finishing");
        Ok(())
    }

    pub fn with(mut self, plugin: Box<dyn Plugin>) -> Self {
        self.plugins.push(plugin);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;
    use actions::handler::MockActionHandler;
    use async_trait::async_trait;
    use client::models::message_body::MessageBody;
    use client::models::message_id::MessageId;
    use client::models::socket_message::{Event, MessageEvent, Payload, SocketMessage};
    use client::socket_listener::SocketModeListener;
    use client::MockSlackClient;
    use plugins::MockPlugin;
    use std::future;

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
                        event: Event::Message(MessageEvent {
                            id: MessageId("fake-id".to_string()),
                            text: Some("this is your secret message".to_string()),
                            user: None,
                            blocks: Some(vec![]),
                            channel: None,
                            channel_type: None,
                        }),
                    },
                })
            } else {
                Ok(SocketMessage::Disconnect {})
            }
        }
    }

    #[tokio::test]
    async fn disconnect_after_disconnect_message_received() {
        let bot = SlackBot::from(mock_client(), Box::new(MockActionHandler::new()));

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn forward_event_message_to_plugin() {
        let mut mock_action_handler = Box::new(MockActionHandler::new());
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin
            .expect_on_event()
            .times(1)
            .returning(|_| Box::pin(future::ready(Action::DoNothing)));
        mock_action_handler
            .expect_handle()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(Ok(()))));
        let bot = SlackBot::from(mock_client(), mock_action_handler).with(mock_plugin);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn forward_event_outcome_to_action_handler() {
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin.expect_on_event().returning(|_| {
            Box::pin(future::ready(Action::MessageChannel {
                channel: String::from("my test channel"),
                message: MessageBody::from_text("my test message"),
            }))
        });
        let mut mock_action_handler = Box::new(MockActionHandler::new());
        mock_action_handler
            .expect_handle()
            .times(1)
            .withf(|action, client| match action {
                Action::MessageChannel { channel, message } => {
                    channel == "my test channel" && message.get_text() == "my test message"
                }
                _ => false,
            })
            .returning(|_, _| Box::pin(future::ready(Ok(()))));
        let bot = SlackBot::from(mock_client(), mock_action_handler).with(mock_plugin);

        bot.run().await.unwrap();
    }

    fn mock_client() -> Arc<MockSlackClient> {
        let mut mock_slack_client = MockSlackClient::new();
        mock_slack_client
            .expect_connect_to_socket_mode()
            .times(1)
            .returning(|| Ok(Box::new(TestSocketModeListener::default())));
        Arc::new(mock_slack_client)
    }
}
