use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use slack_morphism::blocks::SlackBlock;

pub mod response;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum SocketMessage {
    #[serde(rename = "hello")]
    Hello {},
    #[serde(rename = "disconnect")]
    Disconnect {},
    #[serde(rename = "events_api")]
    Event {
        envelope_id: String,
        payload: Payload,
    },
    #[serde(rename = "interactive")]
    Interactive { envelope_id: String },
    #[serde(rename = "slash_commands")]
    SlashCommand { envelope_id: String },
}

// Ignores the type field, because it seems to always be `event_callback`
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Payload {
    pub event: Event,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Event {
    #[serde(rename = "event_ts")]
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub text: Option<String>,
    pub user: Option<String>,
    pub blocks: Vec<SlackBlock>,
    pub channel: Option<String>,
    pub channel_type: Option<String>,
}
