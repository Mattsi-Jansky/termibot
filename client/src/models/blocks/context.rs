use crate::models::blocks::elements::image::ImageElement;
use crate::models::blocks::objects::text::Text;
use crate::models::blocks::Block;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ContextBlock {
    pub elements: Vec<ContextElement>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ContextElement {
    Image(ImageElement),
    Text(Text),
}

impl ContextBlock {
    pub fn new(elements: Vec<ContextElement>) -> Self {
        Self { elements }
    }
}

impl ContextElement {
    pub fn markdown(text: &str) -> Self {
        ContextElement::Text(Text::markdown(text))
    }

    pub fn image(url: &str, alt_text: &str) -> Self {
        ContextElement::Image(ImageElement::from_url(url, alt_text))
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

    fn context_json() -> serde_json::Value {
        json!({
            "type": "context",
            "elements": [
                {"type": "image", "image_url": "https://example.com/avatar.png", "alt_text": "Avatar"},
                {"type": "mrkdwn", "text": "*Sam*"}
            ]
        })
    }

    fn context_block() -> Block {
        ContextBlock::new(vec![
            ContextElement::image("https://example.com/avatar.png", "Avatar"),
            ContextElement::markdown("*Sam*"),
        ])
        .into()
    }

    #[test]
    fn should_serialize_context_with_image_and_text() {
        assert_eq!(
            serde_json::to_value(context_block()).unwrap(),
            context_json()
        );
    }

    #[test]
    fn should_parse_context_with_image_and_text() {
        assert_eq!(
            serde_json::from_value::<Block>(context_json()).unwrap(),
            context_block()
        );
    }
}
