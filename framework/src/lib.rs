extern crate core;

use crate::actions::handler::{ActionHandler, DefaultActionHandler};
use client::error::SlackClientError;
use client::models::socket_message::SocketMessage;
use client::{ReqwestSlackClient, SlackClient};
use futures::future::join_all;
use futures::{FutureExt, StreamExt};
use plugins::Plugin;

use std::sync::Arc;

use crate::actions::Action;
use crate::dependencies::DependenciesBuilder;
use tracing::{error, info};

pub mod actions;
pub mod dependencies;
pub mod plugins;

pub struct SlackBot {
    client: Arc<dyn SlackClient + Send + Sync>,
    plugins: Vec<Box<dyn Plugin>>,
    action_handler: Box<dyn ActionHandler>,
    dependencies_builder: DependenciesBuilder,
}

impl SlackBot {
    pub fn new(bot_token: &str, app_token: &str) -> Self {
        Self {
            client: Arc::new(ReqwestSlackClient::new(bot_token, app_token)),
            plugins: vec![],
            action_handler: Box::new(DefaultActionHandler {}),
            dependencies_builder: DependenciesBuilder::default(),
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
            dependencies_builder: DependenciesBuilder::default(),
        }
    }

    pub async fn run(self) -> Result<(), SlackClientError> {
        let mut listener = self.client.connect_to_socket_mode().await?;
        let dependencies = self.dependencies_builder.build();
        info!("Slack bot starting");

        loop {
            let message = listener.next().await?;
            let mut future_actions = vec![];
            info!("Received message: {message:?}");

            match &message {
                SocketMessage::Event {
                    envelope_id: _,
                    payload,
                } => {
                    for plugin in &self.plugins {
                        let action_future = plugin.on_event(&payload.event, &dependencies);
                        future_actions.push(action_future);
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
                SocketMessage::None => { /* Nothing to do */ }
            }

            let actions: Vec<Action> = join_all(future_actions)
                .await
                .into_iter()
                .flatten()
                .collect();
            let results = join_all(
                actions
                    .into_iter()
                    .map(|action| self.action_handler.handle(action, self.client.clone())),
            )
            .await;

            for result in results {
                if let Err(err) = result {
                    error!("Error occurred when trying to execute action: {:?}", err);
                }
            }
        }

        info!("Slack bot finishing");
        Ok(())
    }

    pub fn with_plugin(mut self, plugin: Box<dyn Plugin>) -> Self {
        self.plugins.push(plugin);
        self
    }

    pub fn with_service<T: Send + Sync + 'static>(mut self, service: T) -> Self {
        self.dependencies_builder.add(service);
        self
    }

    pub fn with_dyn_service<T: Send + Sync + 'static + ?Sized>(mut self, service: Box<T>) -> Self {
        self.dependencies_builder.add_dyn(service);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;
    use crate::dependencies::Dependencies;
    use actions::handler::MockActionHandler;
    use async_trait::async_trait;
    use client::models::message_body::MessageBody;
    use client::models::message_id::MessageId;
    use client::models::socket_message::{Event, MessageEvent, Payload, SocketMessage};
    use client::socket_listener::SocketModeListener;
    use client::MockSlackClient;
    use plugins::MockPlugin;
    use std::future;

    #[derive(Default)]
    struct TestSocketModeListener {
        call_count: usize,
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
        let mock_action_handler = Box::new(MockActionHandler::new());
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin
            .expect_on_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));
        let bot = SlackBot::from(mock_client(), mock_action_handler).with_plugin(mock_plugin);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn forward_service_to_plugin() {
        let mock_action_handler = Box::new(MockActionHandler::new());
        let mock_plugin_asserts_dependencies_passed =
            Box::new(MockPluginAssertDependencyIsPassed());
        let bot = SlackBot::from(mock_client(), mock_action_handler)
            .with_service(12)
            .with_plugin(mock_plugin_asserts_dependencies_passed);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn forward_action_to_action_handler() {
        let mut mock_action_handler = Box::new(MockActionHandler::new());
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin.expect_on_event().times(1).returning(|_, _| {
            Box::pin(future::ready(vec![Action::MessageChannel {
                channel: "".to_string(),
                message: MessageBody::from_text("test"),
            }]))
        });
        mock_action_handler
            .expect_handle()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(Ok(()))));
        let bot = SlackBot::from(mock_client(), mock_action_handler).with_plugin(mock_plugin);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn forward_multiple_actions_to_action_handler() {
        let mut mock_action_handler = Box::new(MockActionHandler::new());
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin.expect_on_event().times(1).returning(|_, _| {
            Box::pin(future::ready(vec![
                Action::MessageChannel {
                    channel: "".to_string(),
                    message: MessageBody::from_text("test 1"),
                },
                Action::MessageChannel {
                    channel: "".to_string(),
                    message: MessageBody::from_text("test 2"),
                },
            ]))
        });
        mock_action_handler
            .expect_handle()
            .times(2)
            .returning(|_, _| Box::pin(future::ready(Ok(()))));
        let bot = SlackBot::from(mock_client(), mock_action_handler).with_plugin(mock_plugin);

        bot.run().await.unwrap();
    }

    struct MockPluginAssertDependencyIsPassed();
    #[async_trait]
    impl Plugin for MockPluginAssertDependencyIsPassed {
        async fn on_event(&self, _event: &Event, dependencies: &Dependencies) -> Vec<Action> {
            assert_eq!(12, *dependencies.get::<i32>().unwrap().read().await);
            vec![]
        }
    }

    #[tokio::test]
    async fn forward_event_outcome_to_action_handler() {
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin.expect_on_event().returning(|_, _| {
            Box::pin(future::ready(vec![Action::MessageChannel {
                channel: String::from("my test channel"),
                message: MessageBody::from_text("my test message"),
            }]))
        });
        let mut mock_action_handler = Box::new(MockActionHandler::new());
        mock_action_handler
            .expect_handle()
            .times(1)
            .withf(|action, _client| match action {
                Action::MessageChannel { channel, message } => {
                    channel == "my test channel" && message.get_text() == "my test message"
                }
                _ => false,
            })
            .returning(|_, _| Box::pin(future::ready(Ok(()))));
        let bot = SlackBot::from(mock_client(), mock_action_handler).with_plugin(mock_plugin);

        bot.run().await.unwrap();
    }

    fn mock_client() -> Arc<MockSlackClient> {
        let mut mock_slack_client = MockSlackClient::new();
        mock_slack_client
            .expect_connect_to_socket_mode()
            .times(1)
            .returning(|| Ok(Box::<TestSocketModeListener>::default()));
        Arc::new(mock_slack_client)
    }
}
