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
