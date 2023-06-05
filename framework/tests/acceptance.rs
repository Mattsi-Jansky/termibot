mod test_config;

use std::path::PathBuf;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rvcr::{VCRMiddleware, VCRMode};
use framework::SlackClient;
use crate::test_config::TEST_CONFIG;

#[tokio::test]
async fn should_send_messages_to_channels_and_threads() {
    let mut bundle = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    bundle.push("tests/resources/replay.vcr.json");

    let middleware = VCRMiddleware::try_from(bundle.clone())
        .unwrap()
        .with_mode(VCRMode::Replay);

    let vcr_client: ClientWithMiddleware = ClientBuilder::new(reqwest::Client::new())
        .with(middleware)
        .build();

    let client = SlackClient::with_client(&TEST_CONFIG.bot_token[..], vcr_client);
    let result = client.message_channel("#bots", "foobar").await;
    assert!(result.is_ok());
    let result = result.unwrap();

    let new_message = &format!("replying to {}", result.message.text)[..];
    let result2 = client.message_thread("#bots", &result.message, new_message);
    assert!(result2.await.is_ok());
}
