use crate::models::message_body::MessageBody;
use async_trait::async_trait;
use error::SlackClientError;
use mockall::automock;
use models::http_response::HttpApiResponse;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use secrecy::{ExposeSecret, Secret};
use socket_listener::TungsteniteSocketModeListenerBuilder;
use tracing::info;

use crate::models::message_id::MessageId;
use crate::rate_limiter::RateLimitingMiddleware;
use crate::socket_listener::SocketModeListener;

pub mod error;
pub mod models;
pub mod rate_limiter;
pub mod socket_listener;

/// A client for talking to the Slack API
///
#[automock]
#[async_trait]
pub trait SlackClient {
    async fn message_channel(
        &self,
        channel: &str,
        message: &MessageBody,
    ) -> Result<HttpApiResponse, SlackClientError>;

    /// Send a reply to a thread.
    ///
    /// Threads are specified with `parent`, specifying the message to reply to.
    async fn message_thread(
        &self,
        channel: &str,
        parent: &MessageId,
        message: &MessageBody,
    ) -> Result<HttpApiResponse, SlackClientError>;

    /// Open a Socket Mode connection
    ///
    /// Gets the websocket address from the Slack API and returns a connected `SlackSocketModeListener`.
    async fn connect_to_socket_mode(&self)
        -> Result<Box<dyn SocketModeListener>, SlackClientError>;
}

/// A client for talking to the Slack API
#[derive(Debug)]
pub struct ReqwestSlackClient {
    bot_token: Secret<String>,
    app_token: Secret<String>,
    http: ClientWithMiddleware,
}

impl ReqwestSlackClient {
    pub fn new(bot_token: &str, app_token: &str) -> ReqwestSlackClient {
        ReqwestSlackClient {
            bot_token: Secret::new(String::from(bot_token)),
            app_token: Secret::new(String::from(app_token)),
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
            bot_token: Secret::new(String::from(bot_token)),
            app_token: Secret::new(String::from(app_token)),
            http: client,
        }
    }
}

#[async_trait]
impl SlackClient for ReqwestSlackClient {
    #[tracing::instrument]
    async fn message_channel(
        &self,
        channel: &str,
        body: &MessageBody,
    ) -> Result<HttpApiResponse, SlackClientError> {
        let body = serde_json::json!({
            "channel": channel,
            "text": body.get_text(),
            "blocks": body.get_blocks()
        });
        info!("Messaging channel {} with {:?}", channel, body);

        let result = self
            .http
            .post("https://slack.com/api/chat.postMessage")
            .header(
                "Authorization",
                format!("Bearer {}", self.bot_token.expose_secret()),
            )
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await?
            .json::<HttpApiResponse>()
            .await
            .map_err(SlackClientError::from);

        if result.is_ok() {
            let response = result.unwrap();
            if !response.ok {
                if response.error.is_none() && response.errors.is_none() {
                    Err(SlackClientError(
                        "Slack returned not-okay result but no errors".to_string(),
                    ))
                } else {
                    let err_type = response
                        .error
                        .unwrap_or("No error type provided".to_string());
                    let errors = response
                        .errors
                        .unwrap_or(vec![])
                        .into_iter()
                        .reduce(|acc, err| format!("{},{}", acc, err))
                        .unwrap();
                    Err(SlackClientError(format!("{}: [{}]", err_type, errors)))
                }
            } else {
                Ok(response)
            }
        } else {
            result
        }
    }

    /// Send a reply to a thread.
    ///
    /// Threads are specified with `parent`, specifying the message to reply to.
    #[tracing::instrument]
    async fn message_thread(
        &self,
        channel: &str,
        parent: &MessageId,
        body: &MessageBody,
    ) -> Result<HttpApiResponse, SlackClientError> {
        info!(
            "Messaging channel {}, thread {:?} with {:?}",
            channel, parent, body
        );
        self.http
            .post("https://slack.com/api/chat.postMessage")
            .header(
                "Authorization",
                format!("Bearer {}", self.bot_token.expose_secret()),
            )
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "channel": channel,
                "thread_ts": parent,
                "text": body.get_text()
            }))
            .send()
            .await?
            .json::<HttpApiResponse>()
            .await
            .map_err(SlackClientError::from)
    }

    #[tracing::instrument]
    async fn connect_to_socket_mode(
        &self,
    ) -> Result<Box<dyn SocketModeListener>, SlackClientError> {
        info!("Connecting to socket mode");
        let builder = self
            .http
            .post("https://slack.com/api/apps.connections.open")
            .header(
                "Authorization",
                format!("Bearer {}", self.app_token.expose_secret()),
            )
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
