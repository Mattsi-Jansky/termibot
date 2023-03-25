use slack_morphism::prelude::*;

use crate::config::CONFIG;
use regex::Regex;
use std::sync::Arc;

mod config;

async fn test_interaction_events_function(
    event: SlackInteractionEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    println!("INTERACTION: {:#?}", event);
    Ok(())
}

async fn test_command_events_function(
    event: SlackCommandEvent,
    client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    println!("COMMAND: {:#?}", event);

    let token_value: SlackApiTokenValue = CONFIG.bot_token.clone().into();
    let token: SlackApiToken = SlackApiToken::new(token_value);

    // Sessions are lightweight and basically just a reference to client and token
    let session = client.open_session(&token);

    session
        .api_test(&SlackApiTestRequest::new().with_foo("Test".into()))
        .await?;

    Ok(SlackCommandEventResponse::new(
        SlackMessageContent::new()
            .with_text("Working on it".into())
            .with_blocks(slack_blocks![
                some_into(SlackSectionBlock::new().with_text(md!(
                    "Working section for {}",
                    event.user_id.to_slack_format()
                ))),
                some_into(SlackActionsBlock::new(slack_blocks![
                    some_into(SlackBlockButtonElement::new(
                        "my-simple-action-button".into(),
                        pt!("Action button")
                    )),
                    some_into(
                        SlackBlockStaticSelectElement::new("my-simple-static-menu".into())
                            .with_options(vec![SlackBlockChoiceItem::new(
                                pt!("my-option1"),
                                "my-option1-value".to_string()
                            )])
                    )
                ]))
            ]),
    ))
}

async fn test_push_events_sm_function(
    event: SlackPushEventCallback,
    client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    println!("PUSH: {:#?}", event);
    let result = Ok(());

    // https://open.spotify.com/track/1bCmvezFg5MRcENzCGG1Cy?si=9d043678fb634f08

    match event.event {
        SlackEventCallbackBody::Message(msg) => {
            let content = msg.content.unwrap().text.unwrap();

            let spotify_matcher =
                Regex::new(r"https://open.spotify.com([-a-zA-Z0-9()@:%_\+.~#?&//=]*)*").unwrap();
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
                let message = TestMessageTemplate { url: new_link };
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

    result
}

fn test_error_handler(
    err: Box<dyn std::error::Error + Send + Sync>,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> http::StatusCode {
    eprintln!("ERROR: {:#?}", err);

    // This return value should be OK if we want to return successful ack to the Slack server using Web-sockets
    // https://api.slack.com/apis/connections/socket-implement#acknowledge
    // so that Slack knows whether to retry
    http::StatusCode::OK
}

pub async fn start_bot() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()));

    let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
        .with_command_events(test_command_events_function)
        .with_interaction_events(test_interaction_events_function)
        .with_push_events(test_push_events_sm_function);

    let listener_environment = Arc::new(
        SlackClientEventsListenerEnvironment::new(client.clone())
            .with_error_handler(test_error_handler),
    );

    let socket_mode_listener = SlackClientSocketModeListener::new(
        &SlackClientSocketModeConfig::new(),
        listener_environment.clone(),
        socket_mode_callbacks,
    );

    let app_token_value: SlackApiTokenValue = CONFIG.app_token.clone().into();
    let app_token: SlackApiToken = SlackApiToken::new(app_token_value);

    socket_mode_listener.listen_for(&app_token).await?;
    socket_mode_listener.serve().await;

    Ok(())
}


#[derive(Debug, Clone)]
pub struct TestMessageTemplate {
    pub url: String,
}

impl SlackMessageTemplate for TestMessageTemplate {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new().with_text(format!("/songlink {}", self.url))
    }
}
