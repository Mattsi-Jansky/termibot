use slack_morphism::prelude::*;

use crate::actions::Action;
use crate::actions::Action::*;
use crate::plugins::Plugin;
use async_trait::async_trait;
use lazy_static::lazy_static;
use message_template::SongLinkMessageTemplate;
use regex::Regex;

mod message_template;

lazy_static! {
    static ref SPOTIFY_MATCHER: Regex =
        Regex::new(r"https://open.spotify.com([-a-zA-Z0-9()@:%_\+.~#?&//=]*)*").unwrap();
}
const SONG_LINK_BASE_URL: &str = "https://song.link/s/";

pub struct SongLinkPlugin {}

#[async_trait]
impl Plugin for SongLinkPlugin {
    fn new() -> Self
    where
        Self: Sized,
    {
        SongLinkPlugin {}
    }

    async fn push_event(
        &self,
        event: SlackPushEventCallback,
        _states: SlackClientEventsUserState,
    ) -> Action {
        if let SlackEventCallbackBody::Message(msg) = event.event {
            let content = msg.content.as_ref().unwrap().text.as_ref().unwrap();
            let captures = SPOTIFY_MATCHER.captures(content);

            if let Some(matches) = captures {
                let content = matches
                    .get(0)
                    .expect("regex capture should be present")
                    .as_str();
                let mut new_link = String::from(SONG_LINK_BASE_URL);
                new_link.push_str(&content[31..]);

                let message = SongLinkMessageTemplate { url: new_link };
                ReplyToThread(msg, Box::new(message))
            } else {
                DoNothing
            }
        } else {
            DoNothing
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_locks::RwLock;
    use super::*;

    #[test]
    fn given_event_not_message_do_nothing() {
        let plugin = SongLinkPlugin::new();
        let event = SlackPushEventCallback {
            team_id: SlackTeamId(String::from("teamid")),
            api_app_id: SlackAppId(String::from("appId")),
            event: SlackEventCallbackBody::EmojiChanged(SlackEmojiChangedEvent {
                subtype: SlackEmojiEventType::EmojiRemoved,
                name: None,
                names: None,
                old_name: None,
                new_name: None,
                value: None,
                event_ts: SlackTs(String::from("3M0J1")),
            }),
            event_id: SlackEventId(String::from("event_id")),
            event_time: SlackDateTime(Default::default()),
            event_context: None,
            authed_users: None,
            authorizations: None,
        };
        let states = RwLock::new(SlackClientEventsUserStateStorage::new());

        let result = plugin.push_event(event, states);

        assert!(matches!(DoNothing, result));
    }

    #[test]
    fn given_message_that_does_not_match_known_music_streaming_service_do_nothing() {
        let plugin = SongLinkPlugin::new();
        let event = SlackPushEventCallback {
            team_id: SlackTeamId(String::from("teamid")),
            api_app_id: SlackAppId(String::from("appId")),
            event: SlackEventCallbackBody::Message(SlackMessageEvent {
                origin: SlackMessageOrigin {
                    ts: SlackTs(String::from("M3554G3")),
                    channel: None,
                    channel_type: None,
                    thread_ts: None,
                    client_msg_id: None,
                },
                content: Some(SlackMessageContent {
                    text: Some(String::from("https://termisoc.org/")),
                    blocks: None,
                    attachments: None,
                    upload: None,
                    files: None,
                    reactions: None,
                }),
                sender: SlackMessageSender {
                    user: None,
                    bot_id: None,
                    username: None,
                    display_as_bot: None,
                },
                subtype: None,
                hidden: None,
                edited: None,
                deleted_ts: None,
            }),
            event_id: SlackEventId(String::from("event_id")),
            event_time: SlackDateTime(Default::default()),
            event_context: None,
            authed_users: None,
            authorizations: None,
        };
        let states = RwLock::new(SlackClientEventsUserStateStorage::new());

        let result = plugin.push_event(event, states);

        assert!(matches!(DoNothing, result));
    }
}
