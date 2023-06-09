use serde::Deserialize;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio::net::TcpStream;
use futures_util::StreamExt;
use crate::error::SlackClientError;

#[derive(Debug, Deserialize)]
pub struct SlackSocketModeListenerBuilder {
    /// The websocket address changes per account. You get the URL by requesting it from the `apps.connections.open` endpoint.
    url: String
}

impl SlackSocketModeListenerBuilder {
    pub async fn connect(&self) -> Result<SlackSocketModeListener, SlackClientError> {
        let url = url::Url::parse(&self.url).unwrap();
        let (stream, _) = connect_async(url).await?;
        Ok(SlackSocketModeListener{ stream })
    }
}

#[derive(Debug)]
pub struct SlackSocketModeListener {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>
}

impl SlackSocketModeListener {
    pub async fn next(&mut self) -> tokio_tungstenite::tungstenite::Message {
        self.stream.next().await.unwrap().unwrap()
    }
}
