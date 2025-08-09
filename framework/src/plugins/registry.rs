use crate::enriched_event::{EnrichedEvent};
use crate::plugins::{Plugin, Subscription};
use tracing::{debug, trace};

struct PluginEntry {
    plugin: Box<dyn Plugin>,
    subscriptions: Vec<Subscription>,
}

/// Registry that manages plugins and routes events to them based on subscriptions
pub struct PluginRegistry {
    plugins: Vec<PluginEntry>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let subscriptions = plugin.subscriptions();
        debug!(
            "Registering plugin with {} subscriptions",
            subscriptions.len()
        );
        for (i, sub) in subscriptions.iter().enumerate() {
            trace!(
                "  Subscription {}: pattern={}, description={:?}",
                i,
                sub.command_pattern.as_str(),
                sub.description
            );
        }

        self.plugins.push(PluginEntry {
            plugin,
            subscriptions,
        });
    }

    /// Get all plugins
    pub fn all(&self) -> impl Iterator<Item = &Box<dyn Plugin>> {
        self.plugins.iter().map(|entry| &entry.plugin)
    }

    /// Find all plugins that have subscriptions matching the given enriched event
    pub fn find_matching_plugins(&self, event: &EnrichedEvent) -> Vec<&Box<dyn Plugin>> {
        let command = match event {
            EnrichedEvent::Command(cmd) => &cmd.command,
        };

        self.plugins
            .iter()
            .filter(|entry| {
                if entry.subscriptions.is_empty() {
                    debug!(
                        "Plugin has no subscriptions, skipping for command: {}",
                        command
                    );
                    return false;
                }

                let matches = entry.subscriptions.iter().any(|sub| {
                    let is_match = sub.matches(command);
                    if is_match {
                        debug!(
                            "Subscription matched - pattern: {}, command: {}",
                            sub.command_pattern.as_str(),
                            command
                        );
                    }
                    is_match
                });

                if !matches {
                    debug!("No subscription matched for command: {}", command);
                }

                matches
            })
            .map(|entry| &entry.plugin)
            .collect()
    }

    /// Get a summary of all registered plugins and their subscriptions
    pub fn get_registry_info(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|entry| PluginInfo {
                subscriptions: entry
                    .subscriptions
                    .iter()
                    .map(|sub| SubscriptionInfo {
                        pattern: sub.command_pattern.as_str().to_string(),
                        description: sub.description.clone(),
                    })
                    .collect(),
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub subscriptions: Vec<SubscriptionInfo>,
}

#[derive(Debug, Clone)]
pub struct SubscriptionInfo {
    pub pattern: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Action;
    use crate::dependencies::Dependencies;
    use async_trait::async_trait;
    use client::models::socket_message::{Event};
    use crate::enriched_event::CommandData;

    struct TestPlugin {
        name: String,
        subscriptions: Vec<Subscription>,
    }

    #[async_trait]
    impl Plugin for TestPlugin {
        fn subscriptions(&self) -> Vec<Subscription> {
            self.subscriptions.clone()
        }

        async fn on_enriched_event(
            &self,
            _event: &EnrichedEvent,
            _dependencies: &Dependencies,
        ) -> Vec<Action> {
            vec![]
        }

        async fn on_event(&self, _event: &Event, _dependencies: &Dependencies) -> Vec<Action> {
            vec![]
        }
    }

    #[test]
    fn should_register_plugin() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(TestPlugin {
            name: "test".to_string(),
            subscriptions: vec![Subscription::exact("help")],
        });

        registry.register(plugin);

        assert_eq!(registry.plugins.len(), 1);
    }

    #[test]
    fn should_match_plugins_exact_match() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin {
            name: "help_plugin".to_string(),
            subscriptions: vec![Subscription::exact("help")],
        }));
        registry.register(Box::new(TestPlugin {
            name: "status_plugin".to_string(),
            subscriptions: vec![Subscription::exact("status")],
        }));
        let event = EnrichedEvent::Command(CommandData {
            command: "help".to_string(),
            args: vec![],
            raw_args: String::new(),
            channel: "#general".to_string(),
            user: "U123".to_string(),
        });

        let matching = registry.find_matching_plugins(&event);

        assert_eq!(matching.len(), 1);
    }

    #[test]
    fn should_match_plugins_pattern_match() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin {
            name: "deploy_plugin".to_string(),
            subscriptions: vec![Subscription::pattern(r"^deploy-(prod|staging|dev)$").unwrap()],
        }));
        let event = EnrichedEvent::Command(CommandData {
            command: "deploy-prod".to_string(),
            args: vec![],
            raw_args: String::new(),
            channel: "#general".to_string(),
            user: "U123".to_string(),
        });

        let matching = registry.find_matching_plugins(&event);

        assert_eq!(matching.len(), 1);
    }

    #[test]
    fn given_no_matching_subscriptions_should_return_empty_vec() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin {
            name: "help_plugin".to_string(),
            subscriptions: vec![Subscription::exact("help")],
        }));
        let event = EnrichedEvent::Command(CommandData {
            command: "bannana republic".to_string(),
            args: vec![],
            raw_args: String::new(),
            channel: "#general".to_string(),
            user: "U123".to_string(),
        });

        let matching = registry.find_matching_plugins(&event);

        assert_eq!(matching.len(), 0);
    }

    #[test]
    fn given_no_subscriptions_should_return_empty_vec() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin {
            name: "raw_only".to_string(),
            subscriptions: vec![],
        }));
        let event = EnrichedEvent::Command(CommandData {
            command: "anything".to_string(),
            args: vec![],
            raw_args: String::new(),
            channel: "#general".to_string(),
            user: "U123".to_string(),
        });

        let matching = registry.find_matching_plugins(&event);

        assert_eq!(matching.len(), 0);
    }

    #[test]
    fn given_two_plugins_matching_command_should_return_both() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin {
            name: "general_help".to_string(),
            subscriptions: vec![Subscription::exact("help")],
        }));
        registry.register(Box::new(TestPlugin {
            name: "detailed_help".to_string(),
            subscriptions: vec![Subscription::exact("help")],
        }));
        let event = EnrichedEvent::Command(CommandData {
            command: "help".to_string(),
            args: vec![],
            raw_args: String::new(),
            channel: "#general".to_string(),
            user: "U123".to_string(),
        });

        let matching = registry.find_matching_plugins(&event);

        assert_eq!(matching.len(), 2);
    }

    #[test]
    fn should_get_subscription_information() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin {
            name: "multi_command".to_string(),
            subscriptions: vec![
                Subscription::exact("help").with_description("Show help"),
                Subscription::exact("status").with_description("Show status"),
            ],
        }));

        let info = registry.get_registry_info();

        assert_eq!(info.len(), 1);
        assert_eq!(
            info[0].subscriptions[0].description,
            Some("Show help".to_string())
        );
    }
}
