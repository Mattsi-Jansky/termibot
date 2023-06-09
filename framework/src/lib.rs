use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};
use serde::Deserialize;
use response::ApiResponse;

use crate::message::Message;
use crate::rate_limiter::RateLimitingMiddleware;

mod message;
pub mod rate_limiter;
mod response;

pub struct SlackClient {
    bot_token: String,
    app_token: String,
    http: ClientWithMiddleware,
}

impl SlackClient {
    pub fn new(bot_token: &str, app_token: &str) -> SlackClient {
        SlackClient {
            bot_token: String::from(bot_token),
            app_token: String::from(app_token),
            http: ClientBuilder::new(Client::new())
                .with(RateLimitingMiddleware::new())
                .build(),
        }
    }

    pub fn with_client(bot_token: &str, app_token: &str, client: ClientWithMiddleware) -> SlackClient {
        SlackClient {
            bot_token: String::from(bot_token),
            app_token: String::from(app_token),
            http: client,
        }
    }

    pub async fn message_channel(
        &self,
        channel: &str,
        message: &str,
    ) -> Result<ApiResponse, Error> {
        self.http
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
            .map_err(Error::from)
    }

    pub async fn message_thread(
        &self,
        channel: &str,
        parent: &Message,
        message: &str,
    ) -> Result<Response, Error> {
        self.http
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

    pub async fn connect_to_socket_mode(&self) -> Result<SlackSocketModeListener, Error> {
        self.http
            .post("https://slack.com/api/apps.connections.open")
            .header("Authorization", format!("Bearer {}", self.app_token))
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
            .header("Content-type", "application/x-www-form-urlencoded")
            .send()
            .await?
            .json::<SlackSocketModeListener>()
            .await
            .map_err(Error::from)
    }
}

#[derive(Debug, Deserialize)]
pub struct SlackSocketModeListener {
    /// The websocket address changes per account. You get the URL by requesting it from the `apps.connections.open` endpoint.
    url: String
}
