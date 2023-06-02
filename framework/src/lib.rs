use reqwest::{Client, Error, Response};
use response::ApiResponse;

use serde::Deserialize;
use crate::message::Message;

mod response;
mod message;

pub struct SlackClient {
    bot_token: String,
    client: Client
}

impl SlackClient {
    pub fn new(bot_token: &str) -> SlackClient {
        SlackClient {
            bot_token: String::from(bot_token),
            client: Client::new()
        }
    }

    pub async fn message_channel(&self, channel: &str, message: &str) -> Result<ApiResponse, Error> {
        self.client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "channel": channel,
                "text": message
            }))
            .send()
            .await?
            .json::<ApiResponse>()
            .await
    }

    pub async fn message_thread(&self, channel: &str, parent: &Message, message: &str) -> Result<Response, Error> {
        self.client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "channel": channel,
                "thread_ts": parent.id,
                "text": message
            }))
            .send()
            .await
    }
}
