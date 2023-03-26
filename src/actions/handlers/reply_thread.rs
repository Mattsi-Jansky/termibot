use crate::config::CONFIG;
use slack_morphism::prelude::*;
use std::sync::Arc;

pub async fn reply_to_thread(
    client: &Arc<SlackHyperClient>,
    message_to_reply_to: SlackMessageEvent,
    outgoing_message: Box<dyn SlackMessageTemplate + Send + Sync>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ts = message_to_reply_to.origin.ts.clone();
    let channel = message_to_reply_to.origin.channel.clone().unwrap();
    let token: SlackApiToken = SlackApiToken::new(CONFIG.bot_token.clone());
    let session = client.open_session(&token);

    let request = SlackApiChatPostMessageRequest {
        channel,
        content: outgoing_message.render_template(),
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
