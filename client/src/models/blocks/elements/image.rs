use crate::models::blocks::image::SlackFile;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename = "image")]
pub struct ImageElement {
    pub alt_text: String,
    pub image_url: Option<String>,
    pub slack_file: Option<SlackFile>,
}

impl ImageElement {
    pub fn from_url(url: &str, alt_text: &str) -> Self {
        Self {
            alt_text: alt_text.to_string(),
            image_url: Some(url.to_string()),
            slack_file: None,
        }
    }
}
