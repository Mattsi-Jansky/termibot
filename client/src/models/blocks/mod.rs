pub mod elements;
pub mod text;
pub mod section;
pub mod objects;

use crate::models::blocks::text::RichTextBlock;
use serde::{Deserialize, Serialize};
use crate::models::blocks::section::SectionBlock;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    RichText(RichTextBlock),
    Divider,
    Section(SectionBlock)
}
