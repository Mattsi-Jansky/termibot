use slack_morphism::prelude::*;

use crate::config::CONFIG;
use crate::plugins::songlink::SongLinkPlugin;
use std::sync::Arc;
use crate::core::on_error::on_error;
use crate::plugins::Plugin;
use self::core::{on_interaction, on_push};

mod config;
pub mod plugins;
mod core;
mod on_command;

pub struct SlackBot {
    pub module: Box<dyn Plugin  + Send + Sync>,
}

impl SlackBot {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()));

        let socket_mode_callbacks = SlackSocketModeListenerCallbacks::new()
            .with_command_events(on_command::on_command_event)
            .with_interaction_events(on_interaction::on_interaction_event)
            .with_push_events(on_push::on_push_event);

        let listener_environment = Arc::new(
            SlackClientEventsListenerEnvironment::new(client.clone())
                .with_error_handler(on_error)
                .with_user_state(self),
        );

        let socket_mode_listener = SlackClientSocketModeListener::new(
            &SlackClientSocketModeConfig::new(),
            listener_environment,
            socket_mode_callbacks,
        );

        let app_token_value: SlackApiTokenValue = CONFIG.app_token.clone().into();
        let app_token: SlackApiToken = SlackApiToken::new(app_token_value);

        socket_mode_listener.listen_for(&app_token).await?;
        socket_mode_listener.serve().await;

        Ok(())
    }
}
