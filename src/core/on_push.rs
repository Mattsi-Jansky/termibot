use crate::plugins::EventResponse;
use crate::SlackBot;
use slack_morphism::events::SlackPushEventCallback;
use slack_morphism::hyper_tokio::SlackHyperClient;
use slack_morphism::listener::SlackClientEventsUserState;
use std::sync::Arc;
use slack_morphism::prelude::SlackApiChatPostMessageRequest;
use slack_morphism::SlackApiToken;
use crate::config::CONFIG;

pub async fn on_push_event(
    event: SlackPushEventCallback,
    client: Arc<SlackHyperClient>,
    states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    println!("PUSH: {:#?}", event);
    let inner_state = states.read().await;
    let bot = inner_state.get_user_state::<SlackBot>().unwrap();
    let mut errors = vec![];

    for plugin in bot.plugins.iter() {
        let result = plugin
            .push_event(event.clone(), client.clone(), states.clone())
            .await;

        match result {
            EventResponse::DoNothing => {}
            EventResponse::ReplyToThread(incoming_message_event, outgoing_message) => {
                // client.reply_to_thread(outgoing_message., &incoming_message).await?;
                let ts = incoming_message_event.origin.ts.clone();
                let channel = incoming_message_event.origin.channel.clone().unwrap();
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
            }
            EventResponse::Error(error) => errors.push(error),
        }
    }

    if errors.len() > 0 {
        Err(errors.pop().unwrap())
    } else {
        Ok(())
    }
}
