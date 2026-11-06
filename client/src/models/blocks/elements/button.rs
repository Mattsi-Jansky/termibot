use crate::models::blocks::objects::text::Text;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename = "button")]
pub struct ButtonElement {
    pub text: Text,
    pub url: Option<String>,
    pub action_id: Option<String>,
    pub style: Option<ButtonStyle>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ButtonStyle {
    Primary,
    Danger,
}

impl ButtonElement {
    pub fn link(text: &str, url: &str) -> Self {
        Self {
            text: Text::plain(text),
            url: Some(url.to_string()),
            action_id: None,
            style: None,
        }
    }

    pub fn with_style(mut self, style: ButtonStyle) -> Self {
        self.style = Some(style);
        self
    }
}
