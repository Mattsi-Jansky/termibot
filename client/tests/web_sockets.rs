mod test_client_builder;

use crate::test_client_builder::TestClientBuilder;
use client::SlackClient;
use futures_util::SinkExt;
use slack_morphism::{prelude, slack_blocks, slack_block_item};
use slack_morphism::blocks::SlackBlock;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;
use client::models::{Event, Payload, SocketMessage};

const FAKE_SLACK_TEXT_MESSAGE: &str = "{\"envelope_id\":\"fake-enve-lope-i-d\",\"payload\":{\"token\":\"F4K3T0K3N\",\"team_id\":\"F4K3T34M1D\",\"context_team_id\":\"F4K3T34M1D\",\"context_enterprise_id\":null,\"api_app_id\":\"F4K34P1ID\",\"event\":{\"client_msg_id\":\"fake-client-msg-id\",\"type\":\"message\",\"text\":\"test\",\"user\":\"F4K3USER1D\",\"ts\":\"1686321337.206879\",\"blocks\":[{\"type\":\"rich_text\",\"block_id\":\"\\/5p\",\"elements\":[{\"type\":\"rich_text_section\",\"elements\":[{\"type\":\"text\",\"text\":\"test\"}]}]}],\"team\":\"F4K3T34M1D\",\"channel\":\"F4K3CH4NN3L1D\",\"event_ts\":\"1686321337.206879\",\"channel_type\":\"im\"},\"type\":\"event_callback\",\"event_id\":\"F4K33V3NT1D\",\"event_time\":1686321337,\"authed_users\":[\"F4K3USER1D\"],\"authorizations\":[{\"enterprise_id\":null,\"team_id\":\"F4K3T34M1D\",\"user_id\":\"F4K3USER1D\",\"is_bot\":true,\"is_enterprise_install\":false}],\"is_ext_shared_channel\":false,\"event_context\":\"4-fake-event-context\"},\"type\":\"events_api\",\"accepts_response_payload\":false,\"retry_attempt\":0,\"retry_reason\":\"\"}";
const FAKE_HELLO_MESSAGE: &str = "{\"type\":\"hello\",\"num_connections\":1,\"debug_info\":{\"host\":\"applink-2\",\"build_number\":30,\"approximate_connection_time\":18060},\"connection_info\":{\"app_id\":\"fake-app-id\"}}";

fn start_websockets_server() -> JoinHandle<()> {
    let f = async move {
        let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();
        let (connection, _) = listener.accept().await.expect("No connections to accept");
        let stream = accept_async(connection).await;
        let mut stream = stream.expect("Failed to handshake with connection");
        stream
            .send(Message::Text(String::from(FAKE_HELLO_MESSAGE)))
            .await
            .unwrap();
        stream
            .send(Message::Text(String::from(FAKE_SLACK_TEXT_MESSAGE)))
            .await
            .unwrap();
    };

    tokio::spawn(f)
}

#[tokio::test]
async fn should_initiate_socket_mode_connection() {
    let builder = TestClientBuilder::new("should_initiate_socket_mode_connection");
    let client = builder.new_client();
    let handle = start_websockets_server();
    let mut listener = client.connect_to_socket_mode().await.unwrap();

    let result = listener.next().await;
    assert_eq!(result.unwrap(), SocketMessage::Hello{});
    let result = listener.next().await;
    assert_eq!(result.unwrap(), SocketMessage::Event {envelope_id: String::from("fake-enve-lope-i-d"), payload: Payload { event: Event {
        id: "1686321337.206879".to_string(),
        event_type: "message".to_string(),
        text: Some("test".to_string()),
        user: Some("F4K3USER1D".to_string()),
        blocks: slack_blocks![
            some_into(
                SlackBlock::RichText(serde_json::json!({
                    "block_id": "/5p",
                    "elements": [
                        {
                            "elements": [
                                {
                                    "text": "test",
                                     "type": "text"
                                }
                            ],
                            "type": "rich_text_section"
                        }
                    ]
                }))
                    // .with_block_id("/5p".into())
            )
        ],
        channel: Some("F4K3CH4NN3L1D".to_string()),
        channel_type: Some("im".to_string()),
    } } });
    handle.abort();
}

/*

blocks: [
RichText(Object
{"
    block_id": String("/5p"),
    "elements": Array [
        Object {
            "elements": Array [
                Object {
                    "text": String("test"),
                     "type": String("text")
                }
            ],
            "type": String("rich_text_section")
        }
    ]
})]

 */