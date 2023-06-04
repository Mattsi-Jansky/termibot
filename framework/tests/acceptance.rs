mod test_config;

use framework::SlackClient;
use crate::test_config::TEST_CONFIG;

#[tokio::test]
async fn should_send_messages_to_channels_and_threads() {
    let client = SlackClient::new(&TEST_CONFIG.bot_token[..]);
    let result = client.message_channel("#bots", "foobar").await;
    assert!(result.is_ok());
    let result = result.unwrap();

    let new_message = &format!("replying to {}", result.message.text)[..];
    let result2 = client.message_thread("#bots", &result.message, new_message);
    assert!(result2.await.is_ok());
}
