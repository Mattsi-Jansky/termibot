use serde::Deserialize;

/// Models a message received from the `apps.connections.open` endpoint.
#[derive(Debug, Deserialize)]
pub struct WebsocketUrlMessage {
    /// The websocket address changes per request.
    pub url: String,
}
