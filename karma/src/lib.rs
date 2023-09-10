use async_trait::async_trait;
use client::models::socket_message::Event;
use framework::actions::Action;
use framework::dependencies::Dependencies;
use framework::plugins::Plugin;

pub struct KarmaPlugin {
    upvote_emoji: String,
    downvote_emoji: String,
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
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::models::message_id::MessageId;
    use client::models::socket_message::MessageEvent;
    use framework::dependencies::DependenciesBuilder;
    use regex::internal::Input;

    #[tokio::test]
    async fn given_no_karma_change_do_nothing() {
        let dependencies = DependenciesBuilder::default().build();
        let event = Event::Message(MessageEvent {
            id: MessageId("myMessageId".to_string()),
            text: Some("test message".to_string()),
            user: None,
            blocks: None,
            channel: None,
            channel_type: None,
        });

        let result = KarmaPlugin::default().on_event(&event, &dependencies).await;

        assert_eq!(0, result.len())
    }
}
