use slack_morphism::prelude::*;

use crate::config::CONFIG;
use crate::plugins::Plugin;
use async_trait::async_trait;
use message_template::SongLinkMessageTemplate;
use regex::Regex;
use std::sync::Arc;

mod message_template;

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
                let content = msg.content.unwrap().text.unwrap();

                let spotify_matcher =
                    Regex::new(r"https://open.spotify.com([-a-zA-Z0-9()@:%_\+.~#?&//=]*)*")
                        .unwrap();
                let captures = spotify_matcher.captures(&content);

                if captures.is_some() {
                    let content = captures.unwrap().get(0).unwrap().as_str();
                    let mut new_link = String::from("https://song.link/s/");
                    new_link.push_str(&content[31..]);
                    println!(
                        "================= I HEARD SPOTIFY! NEW LINK: {new_link} ================="
                    );
                    let message = SongLinkMessageTemplate { url: new_link };
                    let ts = msg.origin.ts;
                    let channel = msg.origin.channel.unwrap();

                    Self::reply_to_thread(client, message, ts, channel).await?;
                } else {
                    println!("================= DID NOT REACT TO {content} =================")
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
        ts: SlackTs,
        channel: SlackChannelId,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let token_value: SlackApiTokenValue = CONFIG.bot_token.clone().into();
        let token: SlackApiToken = SlackApiToken::new(token_value);

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
