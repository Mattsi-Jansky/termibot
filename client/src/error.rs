
use tokio::io;
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub struct SlackClientError(String);

impl From<reqwest_middleware::Error> for SlackClientError {
    fn from(value: reqwest_middleware::Error) -> Self {
        SlackClientError(format!(
            "Error with Reqest (HTTP) middleware (Rate limiting?) error: {}",
            value
        ))
    }
}

impl From<reqwest::Error> for SlackClientError {
    fn from(value: reqwest::Error) -> Self {
        SlackClientError(format!("Reqwest (HTTP) error: {}", value))
    }
}

impl From<io::Error> for SlackClientError {
    fn from(value: io::Error) -> Self {
        SlackClientError(format!("IO (TCP?) error: {}", value))
    }
}

impl From<tungstenite::Error> for SlackClientError {
    fn from(value: tungstenite::Error) -> Self {
        SlackClientError(format!("Tungstenite (Websockets) error: {}", value))
    }
}

impl From<serde_json::error::Error> for SlackClientError {
    fn from(value: serde_json::error::Error) -> Self {
        SlackClientError(format!("Serde ([de]serialization) error: {}", value))
    }
}
