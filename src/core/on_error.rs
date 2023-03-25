use std::sync::Arc;
use slack_morphism::hyper_tokio::SlackHyperClient;
use slack_morphism::listener::SlackClientEventsUserState;

pub fn on_error(
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
