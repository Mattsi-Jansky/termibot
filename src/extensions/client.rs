use crate::config::CONFIG;
use async_trait::async_trait;
use slack_morphism::prelude::*;
use slack_morphism::SlackMessageTemplate;
use std::error::Error;

#[async_trait]
pub trait MessageSender {
    async fn reply_to_thread<T: SlackMessageTemplate + Send + Sync>(
        &self,
        message: T,
        event: &SlackMessageEvent,
    ) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl MessageSender for SlackHyperClient {
    async fn reply_to_thread<T: SlackMessageTemplate + Send + Sync>(
        &self,
        message: T,
        event: &SlackMessageEvent,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ts = event.origin.ts.clone();
        let channel = event.origin.channel.clone().unwrap();
        let token: SlackApiToken = SlackApiToken::new(CONFIG.bot_token.clone());
        let session = self.open_session(&token);

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
