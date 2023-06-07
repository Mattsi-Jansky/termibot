mod test_client_builder;

use crate::test_client_builder::TestClientBuilder;
use std::time::SystemTime;

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

//Parameterise the limits so that we can reduce the limit in the test, make it faster
//Check time before and after, make sure minimum time has passed
#[tokio::test]
async fn given_too_many_requests_should_throttle_to_avoid_rate_limit() {
    let builder =
        TestClientBuilder::new("given_too_many_requests_should_throttle_to_avoid_rate_limit");
    let client = builder.new_client();

    let before = SystemTime::now();
    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");
    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");
    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");
    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");
    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");
    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");
    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");
    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");

    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");

    let duration = before.elapsed().unwrap().as_millis();
    assert!(duration >= 75, "Wrong duration: {}", duration);
}
