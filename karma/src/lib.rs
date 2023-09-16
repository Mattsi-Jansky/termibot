use async_trait::async_trait;
use client::models::message_body::MessageBody;
use client::models::socket_message::Event;
use framework::actions::Action;
use framework::dependencies::Dependencies;
use framework::plugins::Plugin;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref KARMA_MATCHER: Regex = Regex::new(r"([^\`\s]{2,})(--|\+\+)(^|\s|$)").unwrap();
}

pub struct KarmaPlugin {
    upvote_emoji: String,
    downvote_emoji: String,
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

        if let Event::Message(message) = event {
            let text = message.text.clone().unwrap_or(String::new());

            for capture in KARMA_MATCHER.captures_iter(&text[..]) {
                let capture = capture.get(0).unwrap().as_str();
                let thing = &capture[..capture.len() - 2];
                let emoji = &self.upvote_emoji;
                results.push(Action::MessageChannel {
                    channel: "".to_string(),
                    message: MessageBody::from_text(&format!(":{emoji}: {thing}: 1")[..]),
                });
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::models::message_body::MessageBody;
    use client::models::message_id::MessageId;
    use client::models::socket_message::MessageEvent;
    use framework::dependencies::DependenciesBuilder;

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

    #[tokio::test]
    async fn given_karma_change_should_return_karma_changed_message() {
        let dependencies = DependenciesBuilder::default().build();
        let event = Event::Message(MessageEvent {
            id: MessageId("myMessageId".to_string()),
            text: Some("sunnydays++".to_string()),
            user: None,
            blocks: None,
            channel: None,
            channel_type: None,
        });

        let result = KarmaPlugin::default().on_event(&event, &dependencies).await;

        dbg!(&result);
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
        let dependencies = DependenciesBuilder::default().build();
        let event = Event::Message(MessageEvent {
            id: MessageId("myMessageId".to_string()),
            text: Some("sunnydays++".to_string()),
            user: None,
            blocks: None,
            channel: None,
            channel_type: None,
        });

        let result = KarmaPlugin::new("up_custom", "down_custom")
            .on_event(&event, &dependencies)
            .await;

        dbg!(&result);
        assert_eq!(1, result.len());
        assert_eq!(
            &Action::MessageChannel {
                channel: "".to_string(),
                message: MessageBody::from_text(":up_custom: sunnydays: 1"),
            },
            result.get(0).unwrap()
        );
    }
}
