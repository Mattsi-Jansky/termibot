use tokio::io;
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub struct SlackClientError {
    error: String
}

impl From<reqwest_middleware::Error> for SlackClientError {
    fn from(value: reqwest_middleware::Error) -> Self {
        SlackClientError { error: format!("Error with Reqest (HTTP) middleware (Rate limiting?) error: {}", value.to_string()) }
    }
}

impl From<reqwest::Error> for SlackClientError {
    fn from(value: reqwest::Error) -> Self {
        SlackClientError { error: format!("Reqwest (HTTP) error: {}", value.to_string()) }
    }
}

impl From<io::Error> for SlackClientError {
    fn from(value: io::Error) -> Self {
        SlackClientError { error: format!("IO (TCP?) error: {}", value.to_string()) }
    }
}

impl From<tungstenite::Error> for SlackClientError {
    fn from(value: tungstenite::Error) -> Self {
        SlackClientError { error: format!("Tungstenite (Websockets) error: {}", value.to_string()) }
    }
}
