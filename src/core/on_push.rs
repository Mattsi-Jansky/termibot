use slack_morphism::events::SlackPushEventCallback;
use std::sync::Arc;
use slack_morphism::hyper_tokio::SlackHyperClient;
use slack_morphism::listener::SlackClientEventsUserState;
use crate::SlackBot;

pub async fn on_push_event(
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
