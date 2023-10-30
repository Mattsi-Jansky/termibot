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
    
    use crate::models::blocks::Block;

    const EMOJI_CHANGELOG_MESSAGE: &str = "[ { \"type\": \"header\", \"text\": { \"type\": \"plain_text\", \"text\": \"Emoji changelog\" } }, { \"type\": \"section\", \"text\": { \"type\": \"mrkdwn\", \"text\": \":heavy_plus_sign: :smile: `:smile:`\\n        By <@userid>\" } } ] ";

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
}
