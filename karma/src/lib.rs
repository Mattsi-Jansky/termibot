use async_trait::async_trait;
use client::models::message_body::MessageBody;
use client::models::socket_message::Event;
use framework::actions::Action;
use framework::dependencies::Dependencies;
use framework::plugins::Plugin;
use lazy_static::lazy_static;
use regex::{CaptureMatches, Regex};
use tracing::error;
use crate::change_request::ChangeRequest;
use crate::services::karma_parser::get_captures;
use crate::services::karma_repository::KarmaRepository;

mod change_request;
pub mod entry;
pub mod services;

pub struct KarmaPlugin {
    upvote_emoji: String,
    downvote_emoji: String
}

impl KarmaPlugin {
    fn new(upvote_emoji: &str, downvote_emoji: &str) -> KarmaPlugin {
        KarmaPlugin {
            upvote_emoji: upvote_emoji.to_string(),
            downvote_emoji: downvote_emoji.to_string(),
        }
    }
}

impl Default for KarmaPlugin {
    fn default() -> Self {
        KarmaPlugin {
            upvote_emoji: String::from("upboat"),
            downvote_emoji: String::from("downboat"),
        }
    }
}

#[async_trait]
impl Plugin for KarmaPlugin {
    async fn on_event(&self, event: &Event, dependencies: &Dependencies) -> Vec<Action> {
        let mut results = vec![];
        if let Some(binding) = dependencies.get_dyn::<dyn KarmaRepository + Send + Sync>() {
            let repo = binding.read().await;
            if let Event::Message(message) = event {
                let text = message.text.clone().unwrap_or(String::new());

                for capture in get_captures(&text) {
                    let capture = capture.get(0).unwrap().as_str();
                    let thing = &capture[..capture.len() - 2];
                    let is_increment = capture[capture.len() - 2..].eq("++");
                    let emoji = if is_increment {
                        &self.upvote_emoji
                    } else {
                        &self.downvote_emoji
                    };
                    let value = if is_increment { 1 } else { -1 };
                    repo.upsert_karma_change(ChangeRequest::new(capture, value)).await;
                    if let Some(value) = repo.get_karma_for(capture).await {
                        let channel = if let Some(channel) = message.channel.clone() {
                            channel
                        } else if let Some(user) = message.user.clone() {
                            user
                        } else {
                            error!("Cannot get channel from message");
                            String::new()
                        };
                        results.push(Action::MessageChannel {
                            channel,
                            message: MessageBody::from_text(&format!(":{emoji}: {thing}: {value}")[..]),
                        });
                    } else {
                        error!("Error getting current karma value, DB could not find entry or failed to connect to DB.");
                    }
                }
            }
        } else {
            error!("Error getting KarmaRepository. Did you forget to add it? Check the README");
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use std::future;
    use super::*;
    use client::models::message_body::MessageBody;
    use framework::dependencies::DependenciesBuilder;
    use crate::services::karma_repository::KarmaRepository;
    use crate::services::karma_repository::MockKarmaRepository;
    use tracing_test::traced_test;

    fn build_mocked_dependencies(mut n: Vec<i64>) -> Dependencies {
        let mut dependencies_builder = DependenciesBuilder::default();
        let mut mock_repo = MockKarmaRepository::new();
        mock_repo.expect_upsert_karma_change()
            .times(n.len())
            .returning(|_| Box::pin(future::ready(())));
        mock_repo.expect_get_karma_for()
            .times(n.len())
            .returning(move |_| Box::pin(future::ready(Some(n.pop().unwrap()))));
        dependencies_builder.add_dyn::<dyn KarmaRepository + Send + Sync>(Box::new(mock_repo));
        dependencies_builder.build()
    }

    #[tokio::test]
    async fn given_no_karma_change_do_nothing() {
        let mut dependencies_builder = DependenciesBuilder::default();
        dependencies_builder.add_dyn::<dyn KarmaRepository + Send + Sync>(Box::new(MockKarmaRepository::new()));
        let dependencies = dependencies_builder.build();
        let event = Event::new_test_text_message("test message");

        let result = KarmaPlugin::default().on_event(&event, &dependencies).await;

        assert_eq!(0, result.len())
    }

    #[traced_test]
    #[tokio::test]
    async fn given_repo_fails_to_get_current_karma_score_should_log_error_and_return_no_actions() {
        let mut dependencies_builder = DependenciesBuilder::default();
        let mut mock_repo = MockKarmaRepository::new();
        mock_repo.expect_upsert_karma_change()
            .times(1)
            .returning(|_| Box::pin(future::ready(())));
        mock_repo.expect_get_karma_for()
            .times(1)
            .returning(move |_| Box::pin(future::ready(None)));
        dependencies_builder.add_dyn::<dyn KarmaRepository + Send + Sync>(Box::new(mock_repo));
        let dependencies = dependencies_builder.build();
        let event = Event::new_test_text_message("sunnydays++");

        let result = KarmaPlugin::default().on_event(&event, &dependencies).await;

        assert_eq!(0, result.len());
        logs_assert(|lines: &[&str]| match lines.len() {
            1 => Ok(()),
            n => Err(format!("Expected one logs, but found {}", n)),
        });
        assert!(logs_contain("Error getting current karma value, DB could not find entry or failed to connect to DB."));
    }

    #[tokio::test]
    async fn given_positive_karma_change_should_return_karma_changed_message_and_record_database_change() {
        let dependencies = build_mocked_dependencies(vec![1]);
        let event = Event::new_test_text_message("sunnydays++");

        let result = KarmaPlugin::default().on_event(&event, &dependencies).await;

        assert_eq!(1, result.len());
        assert_eq!(
            &Action::MessageChannel {
                channel: "".to_string(),
                message: MessageBody::from_text(":upboat: sunnydays: 1"),
            },
            result.get(0).unwrap()
        );
    }

    #[tokio::test]
    async fn given_override_of_upvote_emoji_should_return_karma_changed_message_with_custom_emoji()
    {
        let dependencies = build_mocked_dependencies(vec![1]);
        let event = Event::new_test_text_message("sunnydays++");

        let result = KarmaPlugin::new("up_custom", "down_custom")
            .on_event(&event, &dependencies)
            .await;

        assert_eq!(1, result.len());
        assert_eq!(
            &Action::MessageChannel {
                channel: "".to_string(),
                message: MessageBody::from_text(":up_custom: sunnydays: 1"),
            },
            result.get(0).unwrap()
        );
    }

    #[tokio::test]
    async fn given_negative_karma_change_should_return_karma_changed_message_and_record_database_change() {
        let dependencies = build_mocked_dependencies(vec![-1]);
        let event = Event::new_test_text_message("rainydays--");

        let result = KarmaPlugin::default().on_event(&event, &dependencies).await;

        assert_eq!(1, result.len());
        assert_eq!(
            &Action::MessageChannel {
                channel: "".to_string(),
                message: MessageBody::from_text(":downboat: rainydays: -1"),
            },
            result.get(0).unwrap()
        );
    }

    #[tokio::test]
    async fn given_override_of_downvote_emoji_should_return_karma_changed_message_with_custom_emoji() {
        let dependencies = build_mocked_dependencies(vec![-1]);
        let event = Event::new_test_text_message("rainydays--");

        let result = KarmaPlugin::new("up_custom", "down_custom")
            .on_event(&event, &dependencies)
            .await;

        assert_eq!(1, result.len());
        assert_eq!(
            &Action::MessageChannel {
                channel: "".to_string(),
                message: MessageBody::from_text(":down_custom: rainydays: -1"),
            },
            result.get(0).unwrap()
        );
    }

    #[traced_test]
    #[tokio::test]
    async fn given_repo_dependency_missing_log_error() {
        let dependencies = DependenciesBuilder::default().build();
        let event = Event::new_test_text_message("rainydays--");

        let result = KarmaPlugin::new("up_custom", "down_custom")
            .on_event(&event, &dependencies)
            .await;

        assert_eq!(0, result.len());
        logs_assert(|lines: &[&str]| match lines.len() {
            1 => Ok(()),
            n => Err(format!("Expected one logs, but found {}", n)),
        });
        assert!(logs_contain("Error getting KarmaRepository. Did you forget to add it? Check the README"));
    }
}
