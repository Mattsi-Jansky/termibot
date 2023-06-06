mod test_client_builder;

use crate::test_client_builder::TestClientBuilder;

#[tokio::test]
async fn should_send_messages_to_channels_and_threads() {
    let builder = TestClientBuilder::new("should_send_messages_to_channels_and_threads");
    let client = builder.new_client();
    let result = client.message_channel("#bots", "foobar").await;
    assert!(result.is_ok());
    let result = result.unwrap();

    let new_message = &format!("replying to {}", result.message.text)[..];
    let result2 = client.message_thread("#bots", &result.message, new_message).await;
    assert!(result2.is_ok());
}
