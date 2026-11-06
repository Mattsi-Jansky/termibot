extern crate core;

use crate::actions::handler::{ActionHandler, DefaultActionHandler};
use client::error::SlackClientError;
use client::models::socket_message::{Event, SocketMessage};
use client::{ReqwestSlackClient, SlackClient};
use futures::future::join_all;
use plugins::Plugin;

use std::sync::Arc;

use crate::actions::Action;
use crate::dependencies::{Dependencies, DependenciesBuilder};
use crate::event_processor::EventProcessor;
use crate::plugins::registry::PluginRegistry;
use client::socket_listener::{SocketModeListener, TungsteniteSocketModeListener};
use tokio::task::{JoinError, JoinSet};
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
    action_handler: Box<dyn ActionHandler + Send + Sync>,
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
        handler: Box<dyn ActionHandler + Send + Sync>,
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

        let listener = match self.listener {
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
        info!("Slack bot starting");
        log_registered_plugins(&self.plugin_registry);

        let dispatcher = Arc::new(EventDispatcher {
            bot_user_id: identity.user_id.clone(),
            event_processor: EventProcessor::new(identity.user, identity.user_id),
            plugin_registry: self.plugin_registry,
            action_handler: self.action_handler,
            client: self.client,
            dependencies: self.dependencies_builder.build(),
        });
        let result = run_event_loop(listener, dispatcher).await;

        info!("Slack bot finishing");
        result
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

fn log_registered_plugins(plugin_registry: &PluginRegistry) {
    let registry_info = plugin_registry.get_registry_info();
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
}

/// Receives messages from Slack until it disconnects or the connection fails. Each event is
/// handled in its own task, so a slow plugin doesn't hold up the events that arrive after it.
/// Before returning, waits for events still in progress, so their replies aren't dropped.
async fn run_event_loop(
    mut listener: Box<dyn SocketModeListener + Send + Sync>,
    dispatcher: Arc<EventDispatcher>,
) -> Result<(), SlackClientError> {
    let mut in_flight = JoinSet::new();

    let result = loop {
        let message = match listener.next().await {
            Ok(message) => message,
            Err(err) => break Err(SlackClientError::from(err)),
        };
        info!("Received message: {message:?}");

        // Collect finished tasks, so the set doesn't grow for as long as the bot runs
        while let Some(finished) = in_flight.try_join_next() {
            log_task_failure(finished);
        }

        match message {
            SocketMessage::Event { payload, .. } => {
                let dispatcher = dispatcher.clone();
                in_flight.spawn(async move { dispatcher.dispatch(payload.event).await });
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
                break Ok(());
            }
        }
    };

    if !in_flight.is_empty() {
        info!("Waiting for {} event(s) to finish", in_flight.len());
    }
    while let Some(finished) = in_flight.join_next().await {
        log_task_failure(finished);
    }

    result
}

/// Routes an event to the plugins and carries out the actions they return. Shared by the tasks
/// handling events.
struct EventDispatcher {
    bot_user_id: String,
    event_processor: EventProcessor,
    plugin_registry: PluginRegistry,
    action_handler: Box<dyn ActionHandler + Send + Sync>,
    client: Arc<dyn SlackClient + Send + Sync>,
    dependencies: Dependencies,
}

impl EventDispatcher {
    async fn dispatch(&self, event: Event) {
        if is_own_message(&event, &self.bot_user_id) {
            debug!("Ignoring message sent by the bot itself");
            return;
        }

        let enriched_event = self.event_processor.process(&event);
        let mut future_actions = vec![];

        if let Some(ref enriched) = &enriched_event {
            debug!("Successfully enriched event: {:?}", enriched);
            let matching_plugins = self.plugin_registry.find_matching_plugins(enriched);

            if !matching_plugins.is_empty() {
                info!(
                    "Found {} plugin(s) matching enriched event",
                    matching_plugins.len()
                );

                for plugin in matching_plugins {
                    let action_future = plugin.on_enriched_event(enriched, &self.dependencies);
                    future_actions.push(action_future);
                }
            } else {
                debug!("No plugins subscribed to this enriched event");
            }
        } else {
            debug!("Event was not enriched (bot not addressed or not a message)");
        }

        for plugin in self.plugin_registry.all() {
            let action_future = plugin.on_event(&event, &self.dependencies);
            future_actions.push(action_future);
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
}

/// Whether the event is a message the bot posted itself. Plugins never see these, so a reply
/// that happens to match a plugin's trigger can't set off another reply.
fn is_own_message(event: &Event, bot_user_id: &str) -> bool {
    matches!(event, Event::Message(message) if message.user.as_deref() == Some(bot_user_id))
}

fn log_task_failure(finished: Result<(), JoinError>) {
    if let Err(err) = finished {
        error!("Event handling task failed: {err}");
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
    use std::collections::VecDeque;
    use std::future;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::sync::Notify;

    const BOT_USER_ID: &str = "U123456";

    #[derive(Default)]
    struct TestSocketModeListener {
        call_count: usize,
        include_bot_mention: bool,
        sent_by_bot: bool,
    }

    impl TestSocketModeListener {
        fn with_bot_mention(mut self) -> Self {
            self.include_bot_mention = true;
            self
        }

        fn sent_by_bot(mut self) -> Self {
            self.sent_by_bot = true;
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
                            user: Some(
                                if self.sent_by_bot {
                                    BOT_USER_ID
                                } else {
                                    "U789"
                                }
                                .to_string(),
                            ),
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

        mock_plugin
            .expect_subscriptions()
            .returning(std::vec::Vec::new);
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

    #[tokio::test]
    async fn messages_sent_by_the_bot_itself_are_not_passed_to_plugins() {
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin
            .expect_subscriptions()
            .returning(|| vec![Subscription::exact("hello")]);
        mock_plugin.expect_on_enriched_event().times(0);
        mock_plugin.expect_on_event().times(0);

        let bot = SlackBot::from(
            mock_client(),
            Box::new(MockActionHandler::new()),
            Box::new(
                TestSocketModeListener::default()
                    .with_bot_mention()
                    .sent_by_bot(),
            ),
        )
        .with_plugin(mock_plugin);

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn slow_event_does_not_hold_up_later_events() {
        let bot = SlackBot::from(
            mock_client(),
            Box::new(MockActionHandler::new()),
            Box::new(ScriptedSocketModeListener::new(vec![
                message_event("wait"),
                message_event("release"),
            ])),
        )
        .with_plugin(Box::new(WaitForReleasePlugin {
            release: Notify::new(),
        }));

        let result = tokio::time::timeout(Duration::from_secs(5), bot.run()).await;

        assert!(
            result.is_ok(),
            "the first event held up the second one, so neither finished"
        );
    }

    #[tokio::test]
    async fn plugin_panic_does_not_stop_later_events_being_handled() {
        let handled = Arc::new(AtomicUsize::new(0));
        let bot = SlackBot::from(
            mock_client(),
            Box::new(MockActionHandler::new()),
            Box::new(ScriptedSocketModeListener::new(vec![
                message_event("boom"),
                message_event("hello"),
            ])),
        )
        .with_plugin(Box::new(PanicOnBoomPlugin {
            handled: handled.clone(),
        }));

        bot.run().await.unwrap();

        assert_eq!(handled.load(Ordering::SeqCst), 1);
    }

    /// Sends the given messages in order, then disconnects.
    struct ScriptedSocketModeListener {
        messages: VecDeque<SocketMessage>,
    }

    impl ScriptedSocketModeListener {
        fn new(messages: Vec<SocketMessage>) -> Self {
            Self {
                messages: messages.into(),
            }
        }
    }

    #[async_trait]
    impl SocketModeListener for ScriptedSocketModeListener {
        async fn next(&mut self) -> serde_json::error::Result<SocketMessage> {
            Ok(self
                .messages
                .pop_front()
                .unwrap_or(SocketMessage::Disconnect {}))
        }
    }

    fn message_event(text: &str) -> SocketMessage {
        SocketMessage::Event {
            envelope_id: "fake-envelope-id".to_string(),
            payload: Payload {
                event: Event::Message(MessageEvent {
                    id: MessageId(format!("fake-id-{text}")),
                    text: Some(text.to_string()),
                    user: Some("U789".to_string()),
                    blocks: Some(vec![]),
                    channel: Some("#general".to_string()),
                    channel_type: Some("channel".to_string()),
                }),
                authorizations: vec![Authorization {
                    user_id: "F4K3U53R1D".to_string(),
                }],
            },
        }
    }

    /// Doesn't finish handling a "wait" message until a "release" message arrives, so it only
    /// finishes at all if events are handled concurrently.
    struct WaitForReleasePlugin {
        release: Notify,
    }

    #[async_trait]
    impl Plugin for WaitForReleasePlugin {
        async fn on_event(&self, event: &Event, _dependencies: &Dependencies) -> Vec<Action> {
            if let Event::Message(message) = event {
                match message.text.as_deref() {
                    Some("wait") => self.release.notified().await,
                    Some("release") => self.release.notify_one(),
                    _ => {}
                }
            }
            vec![]
        }
    }

    struct PanicOnBoomPlugin {
        handled: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl Plugin for PanicOnBoomPlugin {
        async fn on_event(&self, event: &Event, _dependencies: &Dependencies) -> Vec<Action> {
            if let Event::Message(message) = event {
                if message.text.as_deref() == Some("boom") {
                    panic!("plugin failed");
                }
                self.handled.fetch_add(1, Ordering::SeqCst);
            }
            vec![]
        }
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
                    user_id: BOT_USER_ID.to_string(),
                    team_id: "T123456".to_string(),
                    is_enterprise_install: false,
                })
            });

        Arc::new(mock_slack_client)
    }
}
