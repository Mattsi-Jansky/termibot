mod test_client_builder;

use crate::test_client_builder::TestClientBuilder;
use std::time::SystemTime;
use futures_channel::oneshot::{Receiver, Sender};
use futures_util::{AsyncRead, AsyncWrite, SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_tungstenite::{accept_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
#[ignore]
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
#[ignore]
async fn given_too_many_requests_should_throttle_to_avoid_rate_limit() {
    let builder =
        TestClientBuilder::new("given_too_many_requests_should_throttle_to_avoid_rate_limit");
    let client = builder.new_client();

    let before = SystemTime::now();
    for i in 0..8 {
        client
            .message_channel("#bots", "foobar")
            .await
            .expect("Should succeed");
    }

    client
        .message_channel("#bots", "foobar")
        .await
        .expect("Should succeed");

    let duration = before.elapsed().unwrap().as_millis();
    assert!(duration >= 75, "Wrong duration: {}", duration);
}

#[tokio::test]
async fn should_initiate_socket_mode_connection() {
    let builder =
        TestClientBuilder::new("should_initiate_socket_mode_connection");
    let client = builder.new_client();

    let (con_tx, con_rx) = futures_channel::oneshot::channel();

    let handle = start_websockets_server(con_tx);
    let mut listener = client.connect_to_socket_mode().await.unwrap();

    let result = listener.listen().await;
    println!("Result!! {:?}", result);
    let result = listener.listen().await;
    println!("Result!! {:?}", result);
    handle.abort();
}

fn start_websockets_server(con_tx: Sender<()>) -> JoinHandle<()> {
    let f = async move {
        let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
        con_tx.send(()).unwrap();
        let (connection, _) = listener.accept().await.expect("No connections to accept");
        let stream = accept_async(connection).await;
        let mut stream = stream.expect("Failed to handshake with connection");
        stream.send(Message::Text(String::from(FAKE_HELLO_MESSAGE))).await.unwrap();
        stream.send(Message::Text(String::from(FAKE_SLACK_TEXT_MESSAGE))).await.unwrap();;
    };

    tokio::spawn(f)
}

const FAKE_SLACK_TEXT_MESSAGE: &str = "{\"envelope_id\":\"fake-enve-lope-i-d\",\"payload\":{\"token\":\"F4K3T0K3N\",\"team_id\":\"F4K3T34M1D\",\"context_team_id\":\"F4K3T34M1D\",\"context_enterprise_id\":null,\"api_app_id\":\"F4K34P1ID\",\"event\":{\"client_msg_id\":\"fake-client-msg-id\",\"type\":\"message\",\"text\":\"test\",\"user\":\"F4K3USER1D\",\"ts\":\"1686321337.206879\",\"blocks\":[{\"type\":\"rich_text\",\"block_id\":\"\\/5p\",\"elements\":[{\"type\":\"rich_text_section\",\"elements\":[{\"type\":\"text\",\"text\":\"test\"}]}]}],\"team\":\"F4K3T34M1D\",\"channel\":\"F4K3CH4NN3L1D\",\"event_ts\":\"1686321337.206879\",\"channel_type\":\"im\"},\"type\":\"event_callback\",\"event_id\":\"F4K33V3NT1D\",\"event_time\":1686321337,\"authed_users\":[\"F4K3USER1D\"],\"authorizations\":[{\"enterprise_id\":null,\"team_id\":\"F4K3T34M1D\",\"user_id\":\"F4K3USER1D\",\"is_bot\":true,\"is_enterprise_install\":false}],\"is_ext_shared_channel\":false,\"event_context\":\"4-fake-event-context\"},\"type\":\"events_api\",\"accepts_response_payload\":false,\"retry_attempt\":0,\"retry_reason\":\"\"}";

const FAKE_HELLO_MESSAGE: &str = "{\"type\":\"hello\",\"num_connections\":1,\"debug_info\":{\"host\":\"applink-2\",\"build_number\":30,\"approximate_connection_time\":18060},\"connection_info\":{\"app_id\":\"fake-app-id\"}}";