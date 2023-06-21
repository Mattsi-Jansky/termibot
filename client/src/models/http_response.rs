use crate::models::message_id::MessageId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HttpApiResponse {
    pub ok: bool,
    pub message: Message,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Message {
    #[serde(rename = "ts")]
    pub id: MessageId,
    pub text: String,
    pub user: String,
}
