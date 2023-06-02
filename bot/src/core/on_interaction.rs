use slack_morphism::events::SlackInteractionEvent;
use slack_morphism::hyper_tokio::SlackHyperClient;
use slack_morphism::listener::SlackClientEventsUserState;
use std::sync::Arc;

pub async fn on_interaction_event(
    event: SlackInteractionEvent,
    _client: Arc<SlackHyperClient>,
    _states: SlackClientEventsUserState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    #[cfg(debug_assertions)]
    println!("INTERACTION: {:#?}", event);

    Ok(())
}
