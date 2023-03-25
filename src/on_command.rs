use slack_morphism::prelude::*;

use std::sync::Arc;
use crate::config::CONFIG;

pub async fn on_command_event(
    event: SlackCommandEvent,
    client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    println!("COMMAND: {:#?}", event);

    Ok(SlackCommandEventResponse::new(
        SlackMessageContent::new()
            .with_text("ERROR: Not implemented yet".into())
    ))
}
