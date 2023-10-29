use crate::error::SlackClientError;
use crate::models::socket_message::SocketMessage;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
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
        let message = self.stream.next().await.unwrap().unwrap();

        if message.is_ping() {
            self.stream
                .send(Message::Pong("Pong from slackbot".to_string().into_bytes()))
                .await
                .unwrap();
            Ok(SocketMessage::None)
        } else if !message.is_text() {
            error!(
                "Received unexpected non-text message from WSS: {:?}",
                message
            );
            Ok(SocketMessage::None)
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
                            self.stream
                                .send(Message::Text(
                                    json!({ "envelope_id": envelope_id }).to_string(),
                                ))
                                .await
                                .unwrap();
                        }
                        SocketMessage::Interactive { .. } => { /*Not implemented*/ }
                        SocketMessage::SlashCommand { .. } => { /*Not implemented*/ }

                        SocketMessage::Hello { .. } => { /* Does not need to be ACK'd*/ }
                        SocketMessage::Disconnect { .. } => { /* Does not need to be ACK'd*/ }
                        SocketMessage::None => { /* N/A will not happen here */ }
                    }
                    result
                }
                Err(err) => {
                    warn!("Could not parse previous message from Slack (socket mode), probably an unsupported type not yet implemented. Caused by: `{}`", err);
                    self.next().await
                }
            }
        }
    }
}
