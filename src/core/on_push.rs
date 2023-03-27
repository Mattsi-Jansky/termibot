use crate::core::client::SlackBotHyperClient;
use crate::SlackBot;
use slack_morphism::events::SlackPushEventCallback;
use slack_morphism::hyper_tokio::SlackHyperClient;
use slack_morphism::listener::SlackClientEventsUserState;
use std::sync::Arc;

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
    let client = SlackBotHyperClient::new(client);

    for plugin in bot.plugins.iter() {
        let result = plugin.push_event(event.clone(), states.clone()).await;

        bot.action_handler.resolve_action(result, &client, &mut errors).await;
    }

    if !errors.is_empty() {
        Err(errors.pop().unwrap())
    } else {
        Ok(())
    }
}
