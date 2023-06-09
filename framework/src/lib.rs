use futures_util::StreamExt;
use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use error::SlackClientError;
use response::ApiResponse;
use socket_listener::{SlackSocketModeListener, SlackSocketModeListenerBuilder};

use crate::message::Message;
use crate::rate_limiter::RateLimitingMiddleware;

mod message;
pub mod rate_limiter;
mod response;
mod error;
mod socket_listener;

/// A client for talking to the Slack API
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
    ) -> Result<ApiResponse, SlackClientError> {
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
            .map_err(SlackClientError::from)
    }

    /// Send a reply to a thread.
    ///
    /// Threads are specified with `parent`, specifying the message to reply to.
    pub async fn message_thread(
        &self,
        channel: &str,
        parent: &Message,
        message: &str,
    ) -> Result<Response, SlackClientError> {
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
            .map_err(SlackClientError::from)
    }

    pub async fn connect_to_socket_mode(&self) -> Result<SlackSocketModeListener, SlackClientError> {
        let builder = self.http
            .post("https://slack.com/api/apps.connections.open")
            .header("Authorization", format!("Bearer {}", self.app_token))
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
            .header("Content-type", "application/x-www-form-urlencoded")
            .send()
            .await?
            .json::<SlackSocketModeListenerBuilder>()
            .await?;

        Ok(builder.connect().await?)
    }
}
