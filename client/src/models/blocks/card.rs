use crate::models::blocks::elements::button::ButtonElement;
use crate::models::blocks::elements::image::ImageElement;
use crate::models::blocks::objects::text::Text;
use crate::models::blocks::Block;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct CardBlock {
    pub hero_image: Option<ImageElement>,
    pub icon: Option<ImageElement>,
    pub title: Option<Text>,
    pub subtitle: Option<Text>,
    pub body: Option<Text>,
    pub subtext: Option<Text>,
    pub actions: Option<Vec<ButtonElement>>,
}

impl CardBlock {
    pub fn with_hero_image(mut self, image: ImageElement) -> Self {
        self.hero_image = Some(image);
        self
    }

    pub fn with_title(mut self, title: Text) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_subtitle(mut self, subtitle: Text) -> Self {
        self.subtitle = Some(subtitle);
        self
    }

    pub fn with_body(mut self, body: Text) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_actions(mut self, buttons: Vec<ButtonElement>) -> Self {
        self.actions = Some(buttons);
        self
    }
}

impl From<CardBlock> for Block {
    fn from(block: CardBlock) -> Self {
        Block::Card(Box::new(block))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::blocks::elements::button::ButtonStyle;
    use serde_json::json;

    #[test]
    fn should_serialize_card_with_hero_image_text_and_link_button() {
        let block: Block = CardBlock::default()
            .with_hero_image(ImageElement::from_url(
                "https://example.com/thumbnail.png",
                "A cat",
            ))
            .with_title(Text::plain("Video"))
            .with_subtitle(Text::plain("1:21"))
            .with_body(Text::plain("A cat jumping"))
            .with_actions(vec![ButtonElement::link(
                "▶ Watch",
                "https://example.com/cat.mp4",
            )
            .with_style(ButtonStyle::Primary)])
            .into();

        assert_eq!(
            serde_json::to_value(block).unwrap(),
            json!({
                "type": "card",
                "hero_image": {
                    "type": "image",
                    "image_url": "https://example.com/thumbnail.png",
                    "alt_text": "A cat"
                },
                "title": {"type": "plain_text", "text": "Video"},
                "subtitle": {"type": "plain_text", "text": "1:21"},
                "body": {"type": "plain_text", "text": "A cat jumping"},
                "actions": [{
                    "type": "button",
                    "text": {"type": "plain_text", "text": "▶ Watch"},
                    "url": "https://example.com/cat.mp4",
                    "style": "primary"
                }]
            })
        );
    }

    #[test]
    fn should_parse_card_sent_back_by_slack() {
        let block: Block = serde_json::from_value(json!({
            "type": "card",
            "block_id": "abc",
            "title": {"type": "mrkdwn", "text": "Video", "verbatim": false},
            "actions": [{
                "type": "button",
                "action_id": "xyz",
                "text": {"type": "plain_text", "text": "▶ Watch", "emoji": true},
                "url": "https://example.com/cat.mp4"
            }]
        }))
        .unwrap();

        match block {
            Block::Card(card) => {
                assert_eq!(card.title, Some(Text::markdown("Video")));
                assert_eq!(
                    card.actions.unwrap()[0].url.as_deref(),
                    Some("https://example.com/cat.mp4")
                );
            }
            other => panic!("Expected a card, got {other:?}"),
        }
    }
}
