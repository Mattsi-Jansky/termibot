use crate::error::SlackClientError;
use crate::models::SocketMessage;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

/// Build a `SlackSocketModeListener`, given a Websockets URL for it to connect to.
/// Performs the initial WSS handshake and hands the stream to `SlackSocketModeListener`.
///
/// Can be deserialized from Serde.
#[derive(Debug, Deserialize)]
pub struct SlackSocketModeListenerBuilder {
    /// The websocket address changes per account. You get the URL by requesting it from the `apps.connections.open` endpoint.
    url: String,
}

impl SlackSocketModeListenerBuilder {
    pub async fn connect(&self) -> Result<SlackSocketModeListener, SlackClientError> {
        let url = url::Url::parse(&self.url).unwrap();
        let (stream, _) = connect_async(url).await?;
        Ok(SlackSocketModeListener { stream: stream })
    }
}

/// Wraps a Websockets stream, can be polled for messages.
/// Only reads, does not send.
#[derive(Debug)]
pub struct SlackSocketModeListener {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl SlackSocketModeListener {
    pub async fn next(&mut self) -> serde_json::error::Result<SocketMessage> {
        let json = self
            .stream
            .next()
            .await
            .unwrap()
            .unwrap()
            .into_text()
            .unwrap();
        let result = serde_json::from_str(&json);

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
                }
            }
            Err(_) => {}
        }

        result
    }
}
