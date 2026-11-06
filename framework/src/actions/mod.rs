use client::models::message_body::MessageBody;
use client::models::message_id::MessageId;
use std::fmt;

pub mod handler;

#[derive(Debug, PartialEq)]
pub enum Action {
    MessageChannel {
        channel: String,
        message: MessageBody,
    },
    ReplyToThread {
        channel: String,
        thread_id: MessageId,
        message: MessageBody,
    },
    /// Replies with the images shown after the message's own blocks
    ReplyToThreadWithImages {
        channel: String,
        thread_id: MessageId,
        message: MessageBody,
        images: Vec<Image>,
    },
    /// Tries each action in turn, stopping at the first one that succeeds
    FirstSuccessful(Vec<Action>),
    /// Runs each action once the one before it has finished, carrying on if one fails
    InOrder(Vec<Action>),
}

#[derive(Clone, PartialEq)]
pub struct Image {
    pub url: String,
    /// Uploaded to Slack when present. Otherwise, or if the upload fails, Slack loads `url` itself.
    pub content: Option<Vec<u8>>,
    pub alt_text: String,
}

impl Image {
    pub fn new(url: &str, alt_text: &str) -> Self {
        Self {
            url: url.to_string(),
            content: None,
            alt_text: alt_text.to_string(),
        }
    }

    pub fn with_content(mut self, content: Vec<u8>) -> Self {
        self.content = Some(content);
        self
    }
}

impl fmt::Debug for Image {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Image")
            .field("url", &self.url)
            .field(
                "content",
                &self
                    .content
                    .as_ref()
                    .map(|content| format!("{} bytes", content.len())),
            )
            .field("alt_text", &self.alt_text)
            .finish()
    }
}
