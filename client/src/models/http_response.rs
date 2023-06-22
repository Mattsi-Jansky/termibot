use crate::models::message_id::MessageId;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HttpApiResponse {
    pub ok: bool,
    pub message: Option<Message>,
    pub error: Option<String>,
    pub errors: Option<Vec<String>>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Message {
    #[serde(rename = "ts")]
    pub id: MessageId,
    pub text: String,
    pub user: String,
}
