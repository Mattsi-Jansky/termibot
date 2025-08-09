extern crate core;

use crate::actions::handler::{ActionHandler, DefaultActionHandler};
use client::error::SlackClientError;
use client::models::socket_message::SocketMessage;
use client::{ReqwestSlackClient, SlackClient};
use futures::future::join_all;
use plugins::Plugin;

use std::sync::Arc;

use crate::actions::Action;
use crate::dependencies::DependenciesBuilder;
use crate::event_processor::EventProcessor;
use crate::plugins::registry::PluginRegistry;
use client::socket_listener::{SocketModeListener, TungsteniteSocketModeListener};
use tracing::log::warn;
use tracing::{debug, error, info};

pub mod actions;
pub mod dependencies;
pub mod enriched_event;
pub mod event_processor;
pub mod plugins;

pub struct SlackBot {
    client: Arc<dyn SlackClient + Send + Sync>,
    plugin_registry: PluginRegistry,
    action_handler: Box<dyn ActionHandler>,
    dependencies_builder: DependenciesBuilder,
    listener: Option<Box<dyn SocketModeListener + Send + Sync>>,
}

impl SlackBot {
    pub fn new(bot_token: &str, app_token: &str) -> Self {
        let client = Arc::new(ReqwestSlackClient::new(bot_token, app_token));
        Self {
            client,
            plugin_registry: PluginRegistry::new(),
            action_handler: Box::new(DefaultActionHandler {}),
            dependencies_builder: DependenciesBuilder::default(),
            listener: None,
        }
    }

    pub fn from(
        client: Arc<dyn SlackClient + Send + Sync>,
        handler: Box<dyn ActionHandler>,
        listener: Box<dyn SocketModeListener + Send + Sync>,
    ) -> SlackBot {
        SlackBot {
            client,
            plugin_registry: PluginRegistry::new(),
            action_handler: handler,
            dependencies_builder: DependenciesBuilder::default(),
            listener: Some(listener),
        }
    }

    pub async fn run(self) -> Result<(), SlackClientError> {
        let identity = self.client.get_identity().await?;
        let event_processor = EventProcessor::new(identity.user, identity.user_id);

        let mut listener = match self.listener {
            None => Box::new(
                match TungsteniteSocketModeListener::new(self.client.clone()).await {
                    Ok(listener) => listener,
                    Err(err) => {
                        panic!("Failed to start socket mode listener: {:?}", err)
                    }
                },
            ),
            Some(listener) => listener,
        };
        let dependencies = self.dependencies_builder.build();
        info!("Slack bot starting");

        let registry_info = self.plugin_registry.get_registry_info();
        info!("Registered {} plugins", registry_info.len());
        for (i, plugin_info) in registry_info.iter().enumerate() {
            debug!(
                "Plugin {}: {} subscriptions",
                i + 1,
                plugin_info.subscriptions.len()
            );
            for sub in &plugin_info.subscriptions {
                debug!(
                    "  - Pattern: {}, Description: {:?}",
                    sub.pattern, sub.description
                );
            }
        }

        loop {
            let message = listener.next().await?;
            let mut enriched_event;
            let mut future_actions = vec![];
            info!("Received message: {message:?}");

            match &message {
                SocketMessage::Event {
                    envelope_id: _,
                    payload,
                } => {
                    enriched_event = event_processor.process(&payload.event);

                    if let Some(ref enriched) = &enriched_event {
                        debug!("Successfully enriched event: {:?}", enriched);
                        let matching_plugins =
                            self.plugin_registry.find_matching_plugins(enriched);

                        if !matching_plugins.is_empty() {
                            info!(
                                "Found {} plugin(s) matching enriched event",
                                matching_plugins.len()
                            );

                            for plugin in matching_plugins {
                                let action_future =
                                    plugin.on_enriched_event(enriched, &dependencies);
                                future_actions.push(action_future);
                            }
                        } else {
                            debug!("No plugins subscribed to this enriched event");
                        }
                    } else {
                        debug!("Event was not enriched (bot not addressed or not a message)");
                    }

                    for plugin in self.plugin_registry.all() {
                        let action_future = plugin.on_event(&payload.event, &dependencies);
                        future_actions.push(action_future);
                    }
                }
                SocketMessage::Interactive { .. } => {
                    warn!("Received an interactive message but cannot handle interactive events yet, not implemented.")
                }
                SocketMessage::SlashCommand { .. } => {
                    warn!("Received a slash command message but cannot handle slash commands yet, not implemented.")
                }
                SocketMessage::Hello { .. } => { /* Nothing to do */ }
                SocketMessage::Disconnect { .. } => {
                    info!("Disconnect message received");
                    break;
                }
            }

            let actions: Vec<Action> = join_all(future_actions)
                .await
                .into_iter()
                .flatten()
                .collect();
            if !actions.is_empty() {
                debug!("Executing {} action(s)", actions.len());
            }

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
        self.plugin_registry.register(plugin);
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
    
    
    
    use crate::plugins::Subscription;
    use actions::handler::MockActionHandler;
    use async_trait::async_trait;
    use client::models::auth_test_response::AuthTestResponse;
    
    use client::models::message_id::MessageId;
    use client::models::socket_message::{
        Authorization, Event, MessageEvent, Payload, SocketMessage,
    };
    use client::socket_listener::SocketModeListener;
    use client::MockSlackClient;
    use plugins::MockPlugin;
    use std::future;

    #[derive(Default)]
    struct TestSocketModeListener {
        call_count: usize,
        include_bot_mention: bool,
    }

    impl TestSocketModeListener {
        fn with_bot_mention(mut self) -> Self {
            self.include_bot_mention = true;
            self
        }
    }

    #[async_trait]
    impl SocketModeListener for TestSocketModeListener {
        async fn next(&mut self) -> serde_json::error::Result<SocketMessage> {
            self.call_count += 1;

            if self.call_count == 1 {
                Ok(SocketMessage::Hello {})
            } else if self.call_count == 2 {
                let text = if self.include_bot_mention {
                    Some("@testbot hello world".to_string())
                } else {
                    Some("this is your secret message".to_string())
                };

                Ok(SocketMessage::Event {
                    envelope_id: "fake-envelope-id".to_string(),
                    payload: Payload {
                        event: Event::Message(MessageEvent {
                            id: MessageId("fake-id".to_string()),
                            text,
                            user: Some("U789".to_string()),
                            blocks: Some(vec![]),
                            channel: Some("#general".to_string()),
                            channel_type: Some("channel".to_string()),
                        }),
                        authorizations: vec![Authorization {
                            user_id: "F4K3U53R1D".to_string(),
                        }],
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
            Box::new(MockActionHandler::new()),
            Box::<TestSocketModeListener>::default(),
        );

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn plugin_receives_both_raw_and_enriched_events() {
        let mock_action_handler = Box::new(MockActionHandler::new());
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin
            .expect_subscriptions()
            .returning(|| vec![Subscription::exact("hello")]);
        mock_plugin
            .expect_on_enriched_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));

        mock_plugin
            .expect_on_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));
        let bot = SlackBot::from(
            mock_client(),
            mock_action_handler,
            Box::new(TestSocketModeListener::default().with_bot_mention()),
        )
        .with_plugin(mock_plugin);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn plugin_with_no_subscriptions_only_receives_raw_events() {
        let mock_action_handler = Box::new(MockActionHandler::new());
        let mut mock_plugin = Box::new(MockPlugin::new());

        mock_plugin.expect_subscriptions().returning(std::vec::Vec::new);
        mock_plugin.expect_on_enriched_event().times(0);
        mock_plugin
            .expect_on_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));

        let bot = SlackBot::from(
            mock_client(),
            mock_action_handler,
            Box::new(TestSocketModeListener::default().with_bot_mention()),
        )
        .with_plugin(mock_plugin);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn non_addressed_messages_only_trigger_raw_event() {
        let mock_action_handler = Box::new(MockActionHandler::new());
        let mut mock_plugin = Box::new(MockPlugin::new());

        mock_plugin
            .expect_subscriptions()
            .returning(|| vec![Subscription::exact("hello")]);
        mock_plugin.expect_on_enriched_event().times(0);
        mock_plugin
            .expect_on_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));

        let bot = SlackBot::from(
            mock_client(),
            mock_action_handler,
            Box::new(TestSocketModeListener::default()),
        )
        .with_plugin(mock_plugin);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn multiple_plugins_with_same_subscription() {
        let mock_action_handler = Box::new(MockActionHandler::new());

        let mut plugin1 = Box::new(MockPlugin::new());
        plugin1
            .expect_subscriptions()
            .returning(|| vec![Subscription::exact("hello")]);
        plugin1
            .expect_on_enriched_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));
        plugin1
            .expect_on_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));

        let mut plugin2 = Box::new(MockPlugin::new());
        plugin2
            .expect_subscriptions()
            .returning(|| vec![Subscription::exact("hello")]);
        plugin2
            .expect_on_enriched_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));
        plugin2
            .expect_on_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(vec![])));

        let bot = SlackBot::from(
            mock_client(),
            mock_action_handler,
            Box::new(TestSocketModeListener::default().with_bot_mention()),
        )
        .with_plugin(plugin1)
        .with_plugin(plugin2);

        bot.run().await.unwrap();
    }

    fn mock_client() -> Arc<MockSlackClient> {
        let mut mock_slack_client = MockSlackClient::new();

        mock_slack_client
            .expect_get_identity()
            .times(1)
            .returning(|| {
                Ok(AuthTestResponse {
                    ok: true,
                    url: "https://test.slack.com/".to_string(),
                    team: "Test Team".to_string(),
                    user: "testbot".to_string(),
                    user_id: "U123456".to_string(),
                    team_id: "T123456".to_string(),
                    is_enterprise_install: false,
                })
            });

        Arc::new(mock_slack_client)
    }
}
