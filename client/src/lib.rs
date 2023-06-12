use async_trait::async_trait;
use error::SlackClientError;
use reqwest::{Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use response::ApiResponse;
use socket_listener::{TungsteniteSocketModeListenerBuilder};

use crate::message::Message;
use crate::rate_limiter::RateLimitingMiddleware;
use crate::socket_listener::SocketModeListener;

pub mod error;
pub mod message;
pub mod models;
pub mod rate_limiter;
pub mod response;
pub mod socket_listener;

/// A client for talking to the Slack API
///
#[async_trait]
pub trait SlackClient {
    async fn message_channel(
        &self,
        channel: &str,
        message: &str,
    ) -> Result<ApiResponse, SlackClientError>;

    /// Send a reply to a thread.
    ///
    /// Threads are specified with `parent`, specifying the message to reply to.
    async fn message_thread(
        &self,
        channel: &str,
        parent: &Message,
        message: &str,
    ) -> Result<ApiResponse, SlackClientError>;

    /// Open a Socket Mode connection
    ///
    /// Gets the websocket address from the Slack API and returns a connected `SlackSocketModeListener`.
    async fn connect_to_socket_mode(&self) -> Result<Box<dyn SocketModeListener>, SlackClientError>;
}

/// A client for talking to the Slack API
pub struct ReqwestSlackClient {
    bot_token: String,
    app_token: String,
    http: ClientWithMiddleware,
}

impl ReqwestSlackClient {
    pub fn new(bot_token: &str, app_token: &str) -> ReqwestSlackClient {
        ReqwestSlackClient {
            bot_token: String::from(bot_token),
            app_token: String::from(app_token),
            http: ClientBuilder::new(Client::new())
                .with(RateLimitingMiddleware::new())
                .build(),
        }
    }

    pub fn with_client(
        bot_token: &str,
        app_token: &str,
        client: ClientWithMiddleware,
    ) -> ReqwestSlackClient {
        ReqwestSlackClient {
            bot_token: String::from(bot_token),
            app_token: String::from(app_token),
            http: client,
        }
    }
}

#[async_trait]
impl SlackClient for ReqwestSlackClient {
    async fn message_channel(
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
    async fn message_thread(
        &self,
        channel: &str,
        parent: &Message,
        message: &str,
    ) -> Result<ApiResponse, SlackClientError> {
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
            .await?
            .json::<ApiResponse>()
            .await
            .map_err(SlackClientError::from)
    }
    async fn connect_to_socket_mode(&self) -> Result<Box<dyn SocketModeListener>, SlackClientError> {
        let builder = self
            .http
            .post("https://slack.com/api/apps.connections.open")
            .header("Authorization", format!("Bearer {}", self.app_token))
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
            .header("Content-type", "application/x-www-form-urlencoded")
            .send()
            .await?
            .json::<TungsteniteSocketModeListenerBuilder>()
            .await?;

        Ok(Box::new(builder.connect().await?))
    }
}
