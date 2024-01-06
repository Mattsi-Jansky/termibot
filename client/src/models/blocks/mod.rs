pub mod elements;
pub mod objects;
pub mod section;
pub mod text;

use crate::models::blocks::section::SectionBlock;
use crate::models::blocks::text::{HeaderBlock, RichTextBlock};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    RichText(RichTextBlock),
    Divider,
    Section(SectionBlock),
    Header(HeaderBlock),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::models::blocks::elements::emoji::EmojiElement;
    use crate::models::blocks::elements::text::RichTextSectionElement;
    use crate::models::blocks::elements::user::UserElement;
    use crate::models::blocks::elements::BlockElement;
    use crate::models::blocks::objects::text::TextBody;
    use crate::models::blocks::Block;

    const EMOJI_CHANGELOG_MESSAGE: &str = "[ { \"type\": \"header\", \"text\": { \"type\": \"plain_text\", \"text\": \"Emoji changelog\" } }, { \"type\": \"section\", \"text\": { \"type\": \"mrkdwn\", \"text\": \":heavy_plus_sign: :smile: `:smile:`\\n        By <@userid>\" } } ] ";
    const RICH_TEXT_BLOCK_WITH_EMOJI: &str = "[{\"type\":\"rich_text\",\"block_id\":\"+QITd\",\"elements\":[{\"type\":\"rich_text_section\",\"elements\":[{\"type\":\"emoji\",\"name\":\"mild-panic\"},{\"type\":\"text\",\"text\":\"++\"}]}]}]";
    const RICH_TEXT_BLOCK_WITH_USER_LINK: &str = "[{\"type\": \"rich_text\",\"block_id\": \"6aHD1\",\"elements\": [{\"type\": \"rich_text_section\",\"elements\": [{\"type\": \"text\",\"text\": \"Woohay! Nice one \"},{\"type\": \"user\",\"user_id\": \"U2M31DUM6\"},{\"type\": \"text\",\"text\": \" sam++\"}]}]}]";

    #[test]
    fn should_parse_emoji_changelog_message() {
        let result: Vec<Block> = serde_json::from_str(EMOJI_CHANGELOG_MESSAGE).unwrap();

        assert_eq!(
            result,
            vec![
                HeaderBlock::new("Emoji changelog"),
                SectionBlock::new_markdown(
                    ":heavy_plus_sign: :smile: `:smile:`\n        By <@userid>"
                )
            ]
        )
    }

    #[test]
    fn should_parse_emoji_in_rich_text() {
        let result: Vec<Block> = serde_json::from_str(RICH_TEXT_BLOCK_WITH_EMOJI).unwrap();

        assert_eq!(
            result,
            vec![Block::RichText(RichTextBlock {
                elements: vec![BlockElement::RichTextSection(RichTextSectionElement {
                    elements: vec![
                        BlockElement::Emoji(EmojiElement {
                            name: "mild-panic".to_string()
                        }),
                        BlockElement::Text(TextBody {
                            text: "++".to_string()
                        })
                    ],
                })]
            })]
        )
    }

    #[test]
    fn should_parse_user_link_in_rich_text() {
        let result: Vec<Block> = serde_json::from_str(RICH_TEXT_BLOCK_WITH_USER_LINK).unwrap();

        assert_eq!(
            result,
            vec![Block::RichText(RichTextBlock {
                elements: vec![BlockElement::RichTextSection(RichTextSectionElement {
                    elements: vec![
                        BlockElement::Text(TextBody {
                            text: "Woohay! Nice one ".to_string()
                        }),
                        BlockElement::User(UserElement {
                            id: "U2M31DUM6".to_string()
                        }),
                        BlockElement::Text(TextBody {
                            text: " sam++".to_string()
                        })
                    ]
                })]
            })]
        )
    }
}
