use crate::error::SlackClientError;
use crate::models::socket_message::SocketMessage;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{Error, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{error, info, warn};

#[async_trait]
pub trait SocketModeListener {
    async fn next(&mut self) -> serde_json::error::Result<SocketMessage>;
}

/// Build a `SlackSocketModeListener`, given a Websockets URL for it to connect to.
/// Performs the initial WSS handshake and hands the stream to `SlackSocketModeListener`.
///
/// Can be deserialized from Serde.
#[derive(Debug, Deserialize)]
pub struct TungsteniteSocketModeListenerBuilder {
    /// The websocket address changes per account. You get the URL by requesting it from the `apps.connections.open` endpoint.
    url: String,
}

impl TungsteniteSocketModeListenerBuilder {
    pub async fn connect(&self) -> Result<TungsteniteSocketModeListener, SlackClientError> {
        let url = url::Url::parse(&self.url).unwrap();
        let (stream, _) = connect_async(url).await?;
        Ok(TungsteniteSocketModeListener { stream })
    }
}

/// Wraps a Websockets stream, can be polled for messages.
/// Only reads, does not send.
#[derive(Debug)]
pub struct TungsteniteSocketModeListener {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

#[async_trait]
impl SocketModeListener for TungsteniteSocketModeListener {
    async fn next(&mut self) -> serde_json::error::Result<SocketMessage> {
        let mut parsed_message = None;

        loop {
            let message = self.stream.next().await.unwrap().unwrap();

            if message.is_ping() {
                self.stream
                    .send(Message::Pong("Pong from slackbot".to_string().into_bytes()))
                    .await
                    .unwrap();
                continue;
            } else if message.is_close() {
                warn!("Close message received");
                continue;
            } else if !message.is_text() {
                error!(
                    "Received unexpected non-text message from WSS: {:?}",
                    message
                );
                continue;
            } else {
                let text = message.into_text().unwrap();
                info!("Received message {}", text);
                let result = serde_json::from_str(&text);

                match &result {
                    Ok(inner) => {
                        match inner {
                            SocketMessage::Event {
                                envelope_id,
                                payload: _,
                            } => {
                                self.send_ack(envelope_id).await;
                            }
                            SocketMessage::Interactive { envelope_id } => {
                                self.send_ack(envelope_id).await;
                            }
                            SocketMessage::SlashCommand { envelope_id } => {
                                self.send_ack(envelope_id).await;
                            }

                            SocketMessage::Hello { .. } => { /* Does not need to be ACK'd*/ }
                            SocketMessage::Disconnect { .. } => { /* Does not need to be ACK'd*/ }
                        }
                        parsed_message = Some(result);
                        break;
                    }
                    Err(err) => {
                        warn!("Could not parse previous message from Slack (socket mode), probably an unsupported type not yet implemented. Caused by: `{}`", err);
                        parsed_message = Some(self.next().await);
                        break;
                    }
                }
            }
        }

        parsed_message.unwrap()
    }
}

impl TungsteniteSocketModeListener {
    async fn send_ack(&mut self, envelope_id: &String) {
        self.stream
            .send(Message::Text(
                json!({ "envelope_id": envelope_id }).to_string(),
            ))
            .await
            .unwrap();
    }
}
