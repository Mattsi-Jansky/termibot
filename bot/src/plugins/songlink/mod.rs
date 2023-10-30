use async_trait::async_trait;
use client::models::message_body::MessageBody;
use client::models::socket_message::Event;
use framework::actions::Action;
use framework::dependencies::Dependencies;
use framework::plugins::Plugin;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SPOTIFY_MATCHER: Regex =
        Regex::new(r"https://open.spotify.com([-a-zA-Z0-9()@:%_\+.~#?&//=]*)*").unwrap();
}
const SONG_LINK_BASE_URL: &str = "https://song.link/s/";

pub struct SongLinkPlugin {}

#[async_trait]
impl Plugin for SongLinkPlugin {
    async fn on_event(&self, event: &Event, _dependencies: &Dependencies) -> Vec<Action> {
        match event {
            Event::Message(message) => {
                let text = message.text.clone().unwrap_or(String::new());
                let captures = SPOTIFY_MATCHER.captures(&text[..]);

                if let Some(matches) = captures {
                    let content = matches
                        .get(0)
                        .expect("regex capture should be present")
                        .as_str();
                    let mut new_link = String::from(SONG_LINK_BASE_URL);
                    new_link.push_str(&content[31..]);

                    vec![Action::ReplyToThread {
                        channel: message.channel.clone().unwrap_or(String::new()),
                        thread_id: message.id.clone(),
                        message: MessageBody::from_text(&new_link[..]),
                    }]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
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
    async fn given_no_matching_url_do_nothing() {
        let dependencies = DependenciesBuilder::default().build();
        let event = Event::Message(MessageEvent {
            id: MessageId("myMessageId".to_string()),
            text: Some("test message".to_string()),
            user: None,
            blocks: None,
            channel: None,
            channel_type: None,
        });

        let result = SongLinkPlugin {}.on_event(&event, &dependencies).await;

        assert_eq!(0, result.len())
    }

    #[tokio::test]
    async fn given_spotify_link_should_respond_with_songlink_in_thread() {
        let dependencies = DependenciesBuilder::default().build();
        let event = Event::Message(MessageEvent {
            id: MessageId("myMessageId".to_string()),
            text: Some("https://open.spotify.com/track/0mjOx4zUlL5t4rF1xnrfvi".to_string()),
            user: None,
            blocks: None,
            channel: None,
            channel_type: None,
        });

        let mut result = SongLinkPlugin {}.on_event(&event, &dependencies).await;

        assert_eq!(1, result.len());
        assert_eq!(
            Action::ReplyToThread {
                channel: "".to_string(),
                thread_id: "myMessageId".to_string().into(),
                message: MessageBody::from_text("https://song.link/s/0mjOx4zUlL5t4rF1xnrfvi"),
            },
            result.pop().unwrap()
        )
    }
}
