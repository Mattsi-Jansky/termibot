use builder_pattern::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum Text {
    #[serde(rename = "plain_text")]
    PlainText(TextBody),
    #[serde(rename = "mrkdwn")]
    Markdown(TextBody),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct TextBody {
    pub text: String,
}

impl Text {
    pub fn plain(text: &str) -> Self {
        Text::PlainText(TextBody {
            text: text.to_string(),
        })
    }

    pub fn markdown(text: &str) -> Self {
        Text::Markdown(TextBody {
            text: text.to_string(),
        })
    }
}
