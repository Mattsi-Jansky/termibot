use crate::error::SlackClientError;
use crate::models::socket_message::{MaybeRelevantSocketMessage, SocketMessage};
use crate::socket_listener::MaybeRelevantSocketMessage::{Irrelevant, Relevant};
use crate::SlackClient;
use async_timer::oneshot::Timer;
use async_timer::Oneshot;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};

use serde_json::json;
use std::sync::Arc;
use std::time;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{error, info, warn};

#[async_trait]
pub trait SocketModeListener {
    async fn next(&mut self) -> serde_json::error::Result<SocketMessage>;
}

pub struct TungsteniteSocketModeListener {
    client: Arc<dyn SlackClient + Send + Sync>,
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

#[async_trait]
impl SocketModeListener for TungsteniteSocketModeListener {
    async fn next(&mut self) -> serde_json::Result<SocketMessage> {
        let parsed_message;

        loop {
            tokio::select! {
                message = self.try_next() => {
                    match message {
                        Relevant(message) => {
                            parsed_message = Some(message);
                            break;
                        },
                        Irrelevant => { continue; }
                    }
                }
                _timeout = Timer::new(time::Duration::from_secs(15)) => {
                    info!("Timed out awaiting message from Slack API. Restarting Websockets connection.");
                    let stream_result = Self::init_stream(self.client.clone()).await;
                    match stream_result {
                        Ok(stream) => {
                            self.stream = stream;
                            continue;
                        }
                        Err(err) => {
                            panic!("Failed to re-establish Websocket connection after timeout: {:?}", err);
                        }
                    }
                }
            }
        }

        parsed_message.unwrap()
    }
}

impl TungsteniteSocketModeListener {
    pub async fn new(client: Arc<dyn SlackClient + Send + Sync>) -> Result<Self, SlackClientError> {
        Ok(TungsteniteSocketModeListener {
            client: client.clone(),
            stream: Self::init_stream(client).await?,
        })
    }

    async fn init_stream(
        client: Arc<dyn SlackClient + Send + Sync>,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, SlackClientError> {
        let url = client.get_websocket_url().await?;
        let (stream, _) = connect_async(url).await?;
        Ok(stream)
    }

    async fn try_next(&mut self) -> MaybeRelevantSocketMessage {
        let message = self.stream.next().await.unwrap().unwrap();

        if message.is_ping() {
            self.stream
                .send(Message::Pong("Pong from slackbot".to_string().into_bytes()))
                .await
                .unwrap();
            Irrelevant
        } else if message.is_close() {
            warn!("Close message received");
            Irrelevant
        } else if !message.is_text() {
            error!(
                "Received unexpected non-text message from WSS: {:?}",
                message
            );
            Irrelevant
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
                    Relevant(result)
                }
                Err(err) => {
                    warn!("Could not parse previous message from Slack (socket mode), probably an unsupported type not yet implemented. Caused by: `{}`", err);
                    Irrelevant
                }
            }
        }
    }

    async fn send_ack(&mut self, envelope_id: &String) {
        self.stream
            .send(Message::Text(
                json!({ "envelope_id": envelope_id }).to_string(),
            ))
            .await
            .unwrap();
    }
}
