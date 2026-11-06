use crate::models::blocks::objects::text::{Text, TextBody};
use crate::models::blocks::Block;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct ImageBlock {
    pub alt_text: String,
    pub image_url: Option<String>,
    pub slack_file: Option<SlackFile>,
    pub title: Option<Text>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SlackFile {
    pub id: Option<String>,
    pub url: Option<String>,
}

impl ImageBlock {
    pub fn from_url(url: &str, alt_text: &str) -> Self {
        Self {
            alt_text: alt_text.to_string(),
            image_url: Some(url.to_string()),
            slack_file: None,
            title: None,
        }
    }

    pub fn from_slack_file(file_id: &str, alt_text: &str) -> Self {
        Self {
            alt_text: alt_text.to_string(),
            image_url: None,
            slack_file: Some(SlackFile {
                id: Some(file_id.to_string()),
                url: None,
            }),
            title: None,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(Text::PlainText(TextBody {
            text: title.to_string(),
        }));
        self
    }
}

impl From<ImageBlock> for Block {
    fn from(block: ImageBlock) -> Self {
        Block::Image(block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_serialize_image_from_url() {
        let block: Block = ImageBlock::from_url("https://example.com/cat.png", "A cat").into();

        assert_eq!(
            serde_json::to_value(block).unwrap(),
            json!({
                "type": "image",
                "image_url": "https://example.com/cat.png",
                "alt_text": "A cat"
            })
        );
    }

    #[test]
    fn should_serialize_image_from_slack_file_with_title() {
        let block: Block = ImageBlock::from_slack_file("F123", "A cat")
            .with_title("Video")
            .into();

        assert_eq!(
            serde_json::to_value(block).unwrap(),
            json!({
                "type": "image",
                "slack_file": {"id": "F123"},
                "alt_text": "A cat",
                "title": {"type": "plain_text", "text": "Video"}
            })
        );
    }

    #[test]
    fn should_parse_image_block_sent_back_by_slack() {
        let block: Block = serde_json::from_value(json!({
            "type": "image",
            "block_id": "abc",
            "image_url": "https://example.com/cat.png",
            "alt_text": "A cat",
            "image_width": 640,
            "image_height": 480
        }))
        .unwrap();

        assert_eq!(
            block,
            ImageBlock::from_url("https://example.com/cat.png", "A cat").into()
        );
    }
}
