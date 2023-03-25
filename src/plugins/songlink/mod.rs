use slack_morphism::prelude::*;

use crate::extensions::client::MessageSender;
use crate::plugins::Plugin;
use async_trait::async_trait;
use lazy_static::lazy_static;
use message_template::SongLinkMessageTemplate;
use regex::Regex;
use std::sync::Arc;

mod message_template;

lazy_static! {
    static ref SPOTIFY_MATCHER: Regex =
        Regex::new(r"https://open.spotify.com([-a-zA-Z0-9()@:%_\+.~#?&//=]*)*").unwrap();
}
const SONG_LINK_BASE_URL: &'static str = "https://song.link/s/";

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
        client: Arc<SlackHyperClient>,
        _states: SlackClientEventsUserState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match event.event {
            SlackEventCallbackBody::Message(msg) => {
                let content = msg.content.as_ref().unwrap().text.as_ref().unwrap();
                let captures = SPOTIFY_MATCHER.captures(&content);

                if captures.is_some() {
                    let content = captures
                        .unwrap()
                        .get(0)
                        .expect("regex capture should be present")
                        .as_str();
                    let mut new_link = String::from(SONG_LINK_BASE_URL);
                    new_link.push_str(&content[31..]);

                    let message = SongLinkMessageTemplate { url: new_link };
                    client.reply_to_thread(message, &msg).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
