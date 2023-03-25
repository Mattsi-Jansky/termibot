use slack_morphism::prelude::*;

use crate::config::CONFIG;
use crate::plugins::Plugin;
use async_trait::async_trait;
use regex::Regex;
use std::sync::Arc;

pub struct SongLinkPlugin {}

#[async_trait]
impl Plugin for SongLinkPlugin {
    fn new() -> Self where Self: Sized {
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

                    let token_value: SlackApiTokenValue = CONFIG.bot_token.clone().into();
                    let token: SlackApiToken = SlackApiToken::new(token_value);

                    // Sessions are lightweight and basically just a reference to client and token
                    let session = client.open_session(&token);
                    let message = SongLinkMessageTemplate { url: new_link };
                    let channel = msg.origin.channel.unwrap();
                    let ts = msg.origin.ts;

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

                    // let request = SlackApiChatPostMessageRequest::new(channel, message.render_template(), ts);
                    // SlackApiChatPostMessageResponse::new("wat")

                    session.chat_post_message(&request).await?;
                } else {
                    println!("================= DID NOT REACT TO {content} =================")
                }
            }
            SlackEventCallbackBody::AppHomeOpened(_) => {}
            SlackEventCallbackBody::AppMention(_) => {}
            SlackEventCallbackBody::AppUninstalled(_) => {}
            SlackEventCallbackBody::LinkShared(_) => {}
            SlackEventCallbackBody::EmojiChanged(_) => {}
            SlackEventCallbackBody::MemberJoinedChannel(_) => {}
            SlackEventCallbackBody::MemberLeftChannel(_) => {}
            SlackEventCallbackBody::ChannelCreated(_) => {}
            SlackEventCallbackBody::ChannelDeleted(_) => {}
            SlackEventCallbackBody::ChannelArchive(_) => {}
            SlackEventCallbackBody::ChannelRename(_) => {}
            SlackEventCallbackBody::ChannelUnarchive(_) => {}
            SlackEventCallbackBody::TeamJoin(_) => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SongLinkMessageTemplate {
    pub url: String,
}

impl SlackMessageTemplate for SongLinkMessageTemplate {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new().with_text(format!("{}", self.url))
    }
}
