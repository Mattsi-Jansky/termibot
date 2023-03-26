use crate::config::CONFIG;
use async_trait::async_trait;
use slack_morphism::prelude::*;
use std::sync::Arc;

/// This trait exists to make testing easier. Testing the underlying framework is not viable so we
/// wrap it in a trait that we can mock.
#[async_trait]
pub trait SlackBotClient {
    async fn post_message(
        &self,
        request: &SlackApiChatPostMessageRequest,
    ) -> ClientResult<SlackApiChatPostMessageResponse>;
}

pub struct SlackBotHyperClient {
    hyper_client: Arc<SlackHyperClient>,
    token: SlackApiToken,
}

impl SlackBotHyperClient {
    pub fn new(hyper_client: Arc<SlackHyperClient>) -> Self {
        SlackBotHyperClient {
            hyper_client,
            token: SlackApiToken::new(CONFIG.bot_token.clone()),
        }
    }
}

#[async_trait]
impl SlackBotClient for SlackBotHyperClient {
    async fn post_message(
        &self,
        request: &SlackApiChatPostMessageRequest,
    ) -> ClientResult<SlackApiChatPostMessageResponse> {
        let session = self.hyper_client.open_session(&self.token);
        session.chat_post_message(request).await
    }
}
