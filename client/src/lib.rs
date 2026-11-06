use crate::models::auth_test_response::AuthTestResponse;
use crate::models::file_upload::FileUpload;
use crate::models::message_body::MessageBody;
use crate::models::message_id::MessageId;
use crate::models::upload_url_response::UploadUrlResponse;
use crate::models::websocket_url_message::WebsocketUrlMessage;
use crate::rate_limiter::RateLimitingMiddleware;
use async_trait::async_trait;
use error::SlackClientError;
use mockall::automock;
use models::http_response::HttpApiResponse;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use secrecy::{ExposeSecret, Secret};
use serde_json::{json, Value};
use tracing::info;
use url::Url;

pub mod error;
pub mod models;
pub mod rate_limiter;
pub mod socket_listener;

/// A client for talking to the Slack API
///
#[automock]
#[async_trait]
pub trait SlackClient {
    /// Get the name and id of the bot and the Slack instance
    async fn get_identity(&self) -> Result<AuthTestResponse, SlackClientError>;

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

    /// Get a URL for opening a new Websocket connection
    async fn get_websocket_url(&self) -> Result<Url, SlackClientError>;

    /// Upload a file without sharing it to any channel, returning its file ID
    async fn upload_file(&self, file: FileUpload) -> Result<String, SlackClientError>;
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

    fn api_request(&self, method: &str) -> RequestBuilder {
        self.http
            .post(format!("https://slack.com/api/{method}"))
            .header(
                "Authorization",
                format!("Bearer {}", self.bot_token.expose_secret()),
            )
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
    }

    fn message_json(channel: &str, message: &MessageBody) -> Value {
        let mut body = json!({
            "channel": channel,
            "text": message.get_text(),
            "blocks": message.get_blocks()
        });
        if !message.unfurls() {
            body["unfurl_links"] = json!(false);
            body["unfurl_media"] = json!(false);
        }
        body
    }

    async fn post_message(&self, body: &Value) -> Result<HttpApiResponse, SlackClientError> {
        let result = self
            .api_request("chat.postMessage")
            .json(body)
            .send()
            .await?
            .json::<HttpApiResponse>()
            .await
            .map_err(SlackClientError::from);

        Self::ensure_correct_result_type_because_slack_stupidly_uses_200_status_for_errors(result)
    }

    async fn request_upload_destination(
        &self,
        file: &FileUpload,
    ) -> Result<UploadDestination, SlackClientError> {
        let length = file.content.len().to_string();
        let mut form = vec![
            ("filename", file.filename.as_str()),
            ("length", length.as_str()),
        ];
        if let Some(alt_text) = &file.alt_text {
            form.push(("alt_txt", alt_text.as_str()));
        }

        let response = self
            .api_request("files.getUploadURLExternal")
            .form(&form)
            .send()
            .await?
            .json::<UploadUrlResponse>()
            .await?;

        match (response.ok, response.upload_url, response.file_id) {
            (true, Some(url), Some(file_id)) => Ok(UploadDestination { url, file_id }),
            _ => Err(SlackClientError(format!(
                "Slack didn't provide an upload URL: {}",
                response.error.unwrap_or_default()
            ))),
        }
    }

    async fn complete_upload(&self, file_id: &str) -> Result<(), SlackClientError> {
        let files = json!([{ "id": file_id }]).to_string();
        let result = self
            .api_request("files.completeUploadExternal")
            .form(&[("files", files)])
            .send()
            .await?
            .json::<HttpApiResponse>()
            .await
            .map_err(SlackClientError::from);

        Self::ensure_correct_result_type_because_slack_stupidly_uses_200_status_for_errors(result)
            .map(|_| ())
    }

    fn ensure_correct_result_type_because_slack_stupidly_uses_200_status_for_errors(
        result: Result<HttpApiResponse, SlackClientError>,
    ) -> Result<HttpApiResponse, SlackClientError> {
        if let Ok(response) = result {
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
                        .unwrap_or(String::new());
                    Err(SlackClientError(format!("{}: [{}]", err_type, errors)))
                }
            } else {
                Ok(response)
            }
        } else {
            result
        }
    }
}

#[async_trait]
impl SlackClient for ReqwestSlackClient {
    async fn get_identity(&self) -> Result<AuthTestResponse, SlackClientError> {
        self.api_request("auth.test")
            .send()
            .await?
            .json::<AuthTestResponse>()
            .await
            .map_err(SlackClientError::from)
    }

    #[tracing::instrument]
    async fn message_channel(
        &self,
        channel: &str,
        body: &MessageBody,
    ) -> Result<HttpApiResponse, SlackClientError> {
        let body = Self::message_json(channel, body);
        info!("Messaging channel {} with {:?}", channel, body);

        self.post_message(&body).await
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
        let mut body = Self::message_json(channel, body);
        body["thread_ts"] = json!(parent);

        self.post_message(&body).await
    }

    #[tracing::instrument]
    async fn get_websocket_url(&self) -> Result<Url, SlackClientError> {
        info!("Connecting to socket mode");
        let response = self
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
            .json::<WebsocketUrlMessage>()
            .await?;

        Url::parse(response.url.as_str()).map_err(SlackClientError::from)
    }

    async fn upload_file(&self, file: FileUpload) -> Result<String, SlackClientError> {
        info!("Uploading {:?}", file);
        let destination = self.request_upload_destination(&file).await?;
        self.http
            .post(destination.url)
            .body(file.content)
            .send()
            .await?
            .error_for_status()?;
        self.complete_upload(&destination.file_id).await?;

        Ok(destination.file_id)
    }
}

struct UploadDestination {
    url: String,
    file_id: String,
}
