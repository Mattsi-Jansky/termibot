use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};
use response::ApiResponse;

use serde::Deserialize;
use crate::message::Message;

mod response;
mod message;
pub mod rate_limiter;

pub struct SlackClient {
    bot_token: String,
    client: ClientWithMiddleware
}

impl SlackClient {
    pub fn new(bot_token: &str) -> SlackClient {
        SlackClient {
            bot_token: String::from(bot_token),
            client: ClientBuilder::new(Client::new()).build()
        }
    }

    pub fn with_client(bot_token: &str, client: ClientWithMiddleware) -> SlackClient {
        SlackClient {
            bot_token: String::from(bot_token),
            client
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
            .map_err(|err| Error::from(err))
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
