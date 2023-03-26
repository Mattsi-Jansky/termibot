use crate::actions::handlers::reply_thread::reply_to_thread;
use crate::actions::Action;
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

    for plugin in bot.plugins.iter() {
        let result = plugin
            .push_event(event.clone(), client.clone(), states.clone())
            .await;

        match result {
            Action::DoNothing => {}
            Action::ReplyToThread(incoming_message_event, outgoing_message) => {
                let result =
                    reply_to_thread(&client, incoming_message_event, outgoing_message).await;
                if let Err(error) = result {
                    errors.push(error);
                }
            }
            Action::Error(error) => errors.push(error),
        }
    }

    if errors.len() > 0 {
        Err(errors.pop().unwrap())
    } else {
        Ok(())
    }
}
