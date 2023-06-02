use crate::message::Message;

use serde::Deserialize;

#[derive(Debug,Deserialize)]
pub struct ApiResponse {
    pub ok: bool,
    pub message: Message
}
