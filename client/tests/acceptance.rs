mod test_client_builder;

use crate::test_client_builder::TestClientBuilder;
use std::time::SystemTime;
use futures_util::{AsyncRead, AsyncWrite, SinkExt, StreamExt};
use client::SlackClient;

#[tokio::test]
async fn should_send_messages_to_channels_and_threads() {
    let builder = TestClientBuilder::new("should_send_messages_to_channels_and_threads");
    let client = builder.new_client();

    let result = client.message_channel("#bots", "foobar").await;
    assert!(result.is_ok());
    let result = result.unwrap();

    let new_message = &format!("replying to {}", result.message.text)[..];
    let result2 = client
        .message_thread("#bots", &result.message, new_message)
        .await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn given_too_many_requests_should_throttle_to_avoid_rate_limit() {
    let builder =
        TestClientBuilder::new("given_too_many_requests_should_throttle_to_avoid_rate_limit");
    let client = builder.new_client();

    let before = SystemTime::now();
    for i in 0..9 {
        client
            .message_channel("#bots", "foobar")
            .await
            .expect("Should succeed");
    }

    let duration = before.elapsed().unwrap().as_millis();
    assert!(duration >= 75, "Wrong duration: {}", duration);
}
