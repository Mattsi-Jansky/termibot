use slack_morphism::prelude::*;

use crate::config::CONFIG;
use crate::modules::songlink::SongLinkModule;
use std::sync::Arc;

mod config;
pub mod modules;

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
    states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    println!("PUSH: {:#?}", event);
    let inner_state = states.read().await;
    let bot = inner_state.get_user_state::<SlackBot>().unwrap();

    bot.module
        .push_event(event.clone(), client.clone(), states.clone())
        .await?;

    Ok(())
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

pub struct SlackBot {
    pub module: SongLinkModule,
}

impl SlackBot {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()));

        let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
            .with_command_events(test_command_events_function)
            .with_interaction_events(test_interaction_events_function)
            .with_push_events(test_push_events_sm_function);

        let listener_environment = Arc::new(
            SlackClientEventsListenerEnvironment::new(client.clone())
                .with_error_handler(test_error_handler)
                .with_user_state(self),
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
}
