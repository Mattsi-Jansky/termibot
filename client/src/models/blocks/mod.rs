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
    Header(HeaderBlock)
}

#[cfg(test)]
mod tests {
    use crate::models::blocks::Block;
    use crate::models::blocks::objects::text::{Text, TextBody};
    use super::*;

    const EMOJI_CHANGELOG_MESSAGE: &str = "[ { \"type\": \"header\", \"text\": { \"type\": \"plain_text\", \"text\": \"Emoji changelog\" } }, { \"type\": \"section\", \"text\": { \"type\": \"mrkdwn\", \"text\": \":heavy_plus_sign: :smile: `:smile:`\\n        By <@userid>\" } } ] ";

    #[test]
    fn should_parse_emoji_changelog_message() {
        let result: Vec<Block> = serde_json::from_str(EMOJI_CHANGELOG_MESSAGE).unwrap();

        assert_eq!(
            result,
            vec![
                Block::Header(HeaderBlock { text: Text::PlainText(TextBody { text: "Emoji changelog".to_string() }) }),
                Block::Section(SectionBlock { text: Some(Text::Markdown(TextBody { text: ":heavy_plus_sign: :smile: `:smile:`\n        By <@userid>".to_string() })), fields: None })
            ]
        )
    }
}
