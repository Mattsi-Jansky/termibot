use crate::models::blocks::objects::text::TextBody;
use crate::models::blocks::Block;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ContextBlock {
    pub elements: Vec<ContextElement>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContextElement {
    #[serde(rename = "image")]
    Image(ImageElement),
    #[serde(rename = "mrkdwn")]
    Markdown(TextBody),
    #[serde(rename = "plain_text")]
    PlainText(TextBody),
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ImageElement {
    pub image_url: String,
    pub alt_text: String,
}

impl ContextBlock {
    pub fn new(elements: Vec<ContextElement>) -> Self {
        Self { elements }
    }
}

impl ContextElement {
    pub fn markdown(text: &str) -> Self {
        ContextElement::Markdown(TextBody {
            text: text.to_string(),
        })
    }

    pub fn image(url: &str, alt_text: &str) -> Self {
        ContextElement::Image(ImageElement {
            image_url: url.to_string(),
            alt_text: alt_text.to_string(),
        })
    }
}

impl From<ContextBlock> for Block {
    fn from(block: ContextBlock) -> Self {
        Block::Context(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_serialize_context_with_image_and_text() {
        let block: Block = ContextBlock::new(vec![
            ContextElement::image("https://example.com/avatar.png", "Avatar"),
            ContextElement::markdown("*Sam*"),
        ])
        .into();

        assert_eq!(
            serde_json::to_value(block).unwrap(),
            json!({
                "type": "context",
                "elements": [
                    {"type": "image", "image_url": "https://example.com/avatar.png", "alt_text": "Avatar"},
                    {"type": "mrkdwn", "text": "*Sam*"}
                ]
            })
        );
    }
}
