pub mod registry;

use crate::actions::Action;
use crate::dependencies::Dependencies;
use crate::enriched_event::EnrichedEvent;
use async_trait::async_trait;
use client::models::socket_message::Event;
use mockall::automock;
use regex::Regex;

/// Defines the criteria for which enriched events a plugin wants to receive
#[derive(Debug, Clone)]
pub struct Subscription {
    /// Regex pattern to match against the command
    pub command_pattern: Regex,
    /// Optional description of what this subscription handles (for documentation)
    pub description: Option<String>,
}

impl Subscription {
    /// Create a subscription that matches an exact command
    pub fn exact(command: &str) -> Self {
        Self {
            command_pattern: Regex::new(&format!("^{}$", regex::escape(command)))
                .expect("Failed to create exact match regex"),
            description: None,
        }
    }

    /// Create a subscription that matches commands starting with a prefix
    pub fn prefix(prefix: &str) -> Self {
        Self {
            command_pattern: Regex::new(&format!("^{}", regex::escape(prefix)))
                .expect("Failed to create prefix match regex"),
            description: None,
        }
    }

    /// Create a subscription with a custom regex pattern
    pub fn pattern(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            command_pattern: Regex::new(pattern)?,
            description: None,
        })
    }

    /// Add a description to this subscription
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Check if this subscription matches the given command
    pub fn matches(&self, command: &str) -> bool {
        self.command_pattern.is_match(command)
    }
}

#[async_trait]
#[automock]
pub trait Plugin: Send + Sync {
    /// Returns the list of subscriptions this plugin wants to receive enriched events for
    /// An empty vector means the plugin receives no enriched events
    fn subscriptions(&self) -> Vec<Subscription> {
        vec![]
    }

    /// Called when an enriched event matches one of this plugin's subscriptions
    async fn on_enriched_event(
        &self,
        event: &EnrichedEvent,
        dependencies: &Dependencies,
    ) -> Vec<Action> {
        vec![]
    }

    /// Handle a raw event yourself, and produce actions to perform as a result
    async fn on_event(&self, event: &Event, dependencies: &Dependencies) -> Vec<Action> {
        vec![]
    }
}

#[macro_export]
macro_rules! subscriptions {
    () => {
        vec![]
    };
    ($($pattern:expr),+ $(,)?) => {
        vec![
            $(
                $crate::plugins::Subscription::exact($pattern),
            )+
        ]
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enriched_event::{CommandData, EnrichedEvent};
    
    

    #[test]
    fn should_match_exact_subscription() {
        let sub = Subscription::exact("help");
        assert!(sub.matches("help"));
        assert!(!sub.matches("help2"));
        assert!(!sub.matches("hel"));
        assert!(!sub.matches("help me"));
    }

    #[test]
    fn should_match_prefix_subscription() {
        let sub = Subscription::prefix("deploy");
        assert!(sub.matches("deploy"));
        assert!(sub.matches("deploy-prod"));
        assert!(sub.matches("deployment"));
        assert!(!sub.matches("redeploy"));
    }

    #[test]
    fn should_match_pattern_subscription() {
        let sub = Subscription::pattern(r"^(start|stop|restart)$").unwrap();
        assert!(sub.matches("start"));
        assert!(sub.matches("stop"));
        assert!(sub.matches("restart"));
        assert!(!sub.matches("started"));
        assert!(!sub.matches("stop2"));
    }

    #[test]
    fn should_match_subscription_with_description() {
        let sub = Subscription::exact("help").with_description("Shows help information");
        assert_eq!(sub.description, Some("Shows help information".to_string()));
    }

    struct TestPlugin;

    #[async_trait]
    impl Plugin for TestPlugin {
        fn subscriptions(&self) -> Vec<Subscription> {
            vec![
                Subscription::exact("hello"),
                Subscription::exact("goodbye"),
                Subscription::prefix("echo").with_description("Echoes back the arguments"),
            ]
        }

        async fn on_enriched_event(
            &self,
            event: &EnrichedEvent,
            _dependencies: &Dependencies,
        ) -> Vec<Action> {
            match event {
                EnrichedEvent::Command(cmd) => {
                    if cmd.command == "hello" {
                        vec![Action::MessageChannel {
                            channel: cmd.channel.clone(),
                            message: client::models::message_body::MessageBody::from_text("Hello!"),
                        }]
                    } else {
                        vec![]
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn should_match_plugin_subscriptions() {
        let plugin = TestPlugin;
        let subs = plugin.subscriptions();

        assert_eq!(subs.len(), 3);
        assert!(subs[0].matches("hello"));
        assert!(subs[1].matches("goodbye"));
        assert!(subs[2].matches("echo"));
        assert!(subs[2].matches("echo-test"));
    }

    #[tokio::test]
    async fn should_forward_enriched_event_to_plugin() {
        let plugin = TestPlugin;
        let deps = Dependencies::default();

        let event = EnrichedEvent::Command(CommandData {
            command: "hello".to_string(),
            args: vec![],
            raw_args: String::new(),
            channel: "#general".to_string(),
            user: "U123".to_string(),
        });

        let actions = plugin.on_enriched_event(&event, &deps).await;
        assert_eq!(actions.len(), 1);
    }

    struct MacroTestPlugin;

    #[async_trait]
    impl Plugin for MacroTestPlugin {
        fn subscriptions(&self) -> Vec<Subscription> {
            subscriptions!["help", "status", "ping"]
        }

        async fn on_enriched_event(
            &self,
            _event: &EnrichedEvent,
            _dependencies: &Dependencies,
        ) -> Vec<Action> {
            vec![]
        }
    }

    #[test]
    fn should_match_subscriptions_generated_from_macro() {
        let plugin = MacroTestPlugin;
        let subs = plugin.subscriptions();

        assert_eq!(subs.len(), 3);
        assert!(subs[0].matches("help"));
        assert!(subs[1].matches("status"));
        assert!(subs[2].matches("ping"));
        assert!(!subs[2].matches("pong"));
    }
}
