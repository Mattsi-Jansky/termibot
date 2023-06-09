use tokio::io;
use tokio_tungstenite::tungstenite;

#[derive(Debug)]
pub struct SlackClientError(String);

impl From<reqwest_middleware::Error> for SlackClientError {
    fn from(value: reqwest_middleware::Error) -> Self {
        SlackClientError { 0: format!("Error with Reqest (HTTP) middleware (Rate limiting?) error: {}", value.to_string()) }
    }
}

impl From<reqwest::Error> for SlackClientError {
    fn from(value: reqwest::Error) -> Self {
        SlackClientError { 0: format!("Reqwest (HTTP) error: {}", value.to_string()) }
    }
}

impl From<io::Error> for SlackClientError {
    fn from(value: io::Error) -> Self {
        SlackClientError { 0: format!("IO (TCP?) error: {}", value.to_string()) }
    }
}

impl From<tungstenite::Error> for SlackClientError {
    fn from(value: tungstenite::Error) -> Self {
        SlackClientError { 0: format!("Tungstenite (Websockets) error: {}", value.to_string()) }
    }
}
