mod test_client_builder;

use crate::test_client_builder::TestClientBuilder;
use client::models::blocks::elements::text::{RichTextSectionElement};
use client::models::blocks::elements::BlockElement;
use client::models::blocks::text::RichTextBlock;
use client::models::blocks::Block;
use client::models::socket_message::Event;
use client::models::socket_message::{Payload, SocketMessage};
use client::SlackClient;
use futures_util::{SinkExt, StreamExt};
use serial_test::serial;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, WebSocketStream};
use client::models::blocks::objects::text::TextBody;

const FAKE_SLACK_TEXT_MESSAGE: &str = "{\"envelope_id\":\"fake-enve-lope-i-d\",\"payload\":{\"token\":\"F4K3T0K3N\",\"team_id\":\"F4K3T34M1D\",\"context_team_id\":\"F4K3T34M1D\",\"context_enterprise_id\":null,\"api_app_id\":\"F4K34P1ID\",\"event\":{\"client_msg_id\":\"fake-client-msg-id\",\"type\":\"message\",\"text\":\"test\",\"user\":\"F4K3USER1D\",\"ts\":\"1686321337.206879\",\"blocks\":[{\"type\":\"rich_text\",\"block_id\":\"\\/5p\",\"elements\":[{\"type\":\"rich_text_section\",\"elements\":[{\"type\":\"text\",\"text\":\"test\"}]}]}],\"team\":\"F4K3T34M1D\",\"channel\":\"F4K3CH4NN3L1D\",\"event_ts\":\"1686321337.206879\",\"channel_type\":\"im\"},\"type\":\"event_callback\",\"event_id\":\"F4K33V3NT1D\",\"event_time\":1686321337,\"authed_users\":[\"F4K3USER1D\"],\"authorizations\":[{\"enterprise_id\":null,\"team_id\":\"F4K3T34M1D\",\"user_id\":\"F4K3USER1D\",\"is_bot\":true,\"is_enterprise_install\":false}],\"is_ext_shared_channel\":false,\"event_context\":\"4-fake-event-context\"},\"type\":\"events_api\",\"accepts_response_payload\":false,\"retry_attempt\":0,\"retry_reason\":\"\"}";
const FAKE_HELLO_MESSAGE: &str = "{\"type\":\"hello\",\"num_connections\":1,\"debug_info\":{\"host\":\"applink-2\",\"build_number\":30,\"approximate_connection_time\":18060},\"connection_info\":{\"app_id\":\"fake-app-id\"}}";

async fn start_websocket_server() -> WebSocketStream<TcpStream> {
    let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
    let (connection, _) = listener.accept().await.expect("No connections to accept");
    let stream = accept_async(connection).await;

    stream.expect("Failed to handshake with connection")
}

#[tokio::test]
#[serial]
async fn should_initiate_socket_mode_connection() {
    let websocket_server = async move {
        let mut stream = start_websocket_server().await;
        stream
            .send(Message::Text(String::from(FAKE_HELLO_MESSAGE)))
            .await
            .unwrap();
        stream
            .send(Message::Text(String::from(FAKE_SLACK_TEXT_MESSAGE)))
            .await
            .unwrap();
    };
    let handle = tokio::spawn(websocket_server);
    let builder = TestClientBuilder::new("should_initiate_socket_mode_connection");
    let client = builder.new_client();
    let mut listener = client.connect_to_socket_mode().await.unwrap();

    let result = listener.next().await;
    assert_eq!(result.unwrap(), SocketMessage::Hello {});
    let result = listener.next().await;
    assert_eq!(
        result.unwrap(),
        SocketMessage::Event {
            envelope_id: String::from("fake-enve-lope-i-d"),
            payload: Payload {
                event: Event {
                    id: "1686321337.206879".to_string().into(),
                    event_type: "message".to_string(),
                    text: Some("test".to_string()),
                    user: Some("F4K3USER1D".to_string()),
                    blocks: vec![Block::RichText(
                        RichTextBlock::new()
                            .elements(vec![BlockElement::RichTextSection(
                                RichTextSectionElement::new()
                                    .elements(vec![BlockElement::Text(
                                        TextBody::new().text("test".to_string()).build()
                                    )])
                                    .build()
                            )])
                            .build()
                    )],
                    channel: Some("F4K3CH4NN3L1D".to_string()),
                    channel_type: Some("im".to_string()),
                }
            }
        }
    );
    handle.abort();
}

#[tokio::test]
#[serial]
async fn should_send_acknowledgement_of_event_messages() {
    let (tx, rx) = futures_channel::oneshot::channel();
    let websocket_server = async move {
        let mut stream = start_websocket_server().await;
        stream
            .send(Message::Text(String::from(FAKE_HELLO_MESSAGE)))
            .await
            .unwrap();
        stream
            .send(Message::Text(String::from(FAKE_SLACK_TEXT_MESSAGE)))
            .await
            .unwrap();
        let result = stream.next().await.unwrap();
        tx.send(result).unwrap();
    };
    let handle = tokio::spawn(websocket_server);
    let builder = TestClientBuilder::new("should_send_acknowledgement_of_event_messages");
    let client = builder.new_client();
    let mut listener = client.connect_to_socket_mode().await.unwrap();

    listener.next().await.unwrap();
    listener.next().await.unwrap();

    let acknowledgement = rx.await.unwrap();
    assert_eq!(
        acknowledgement.unwrap(),
        Message::Text("{\"envelope_id\":\"fake-enve-lope-i-d\"}".to_string())
    );
    handle.abort();
}
