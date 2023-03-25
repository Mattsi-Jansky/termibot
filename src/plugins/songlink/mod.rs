use slack_morphism::prelude::*;

use crate::config::CONFIG;
use crate::plugins::Plugin;
use async_trait::async_trait;
use message_template::SongLinkMessageTemplate;
use regex::Regex;
use std::sync::Arc;
use lazy_static::lazy_static;

mod message_template;

lazy_static! {
    static ref SPOTIFY_MATCHER: Regex = Regex::new(r"https://open.spotify.com([-a-zA-Z0-9()@:%_\+.~#?&//=]*)*").unwrap();
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
                    let content = captures.unwrap().get(0).expect("regex capture should be present").as_str();
                    let mut new_link = String::from(SONG_LINK_BASE_URL);
                    new_link.push_str(&content[31..]);

                    let message = SongLinkMessageTemplate { url: new_link };
                    Self::reply_to_thread(client, message, &msg).await?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl SongLinkPlugin {
    async fn reply_to_thread(
        client: Arc<SlackHyperClient>,
        message: SongLinkMessageTemplate,
        event: &SlackMessageEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ts = event.origin.ts.clone();
        let channel = event.origin.channel.clone().unwrap();
        let token: SlackApiToken = SlackApiToken::new(CONFIG.bot_token.clone());

        // Sessions are lightweight and basically just a reference to client and token
        let session = client.open_session(&token);

        let request = SlackApiChatPostMessageRequest {
            channel,
            content: message.render_template(),
            as_user: None,
            icon_emoji: None,
            icon_url: None,
            link_names: None,
            parse: None,
            thread_ts: Some(ts),
            username: None,
            reply_broadcast: None,
            unfurl_links: None,
            unfurl_media: None,
        };

        session.chat_post_message(&request).await?;
        Ok(())
    }
}
