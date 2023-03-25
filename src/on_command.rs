use slack_morphism::prelude::*;

use std::sync::Arc;

pub async fn on_command_event(
    event: SlackCommandEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<SlackCommandEventResponse, Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    println!("COMMAND: {:#?}", event);

    Ok(SlackCommandEventResponse::new(
        SlackMessageContent::new().with_text("ERROR: Not implemented yet".into()),
    ))
}
