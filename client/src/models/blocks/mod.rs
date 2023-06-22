pub mod elements;
pub mod objects;
pub mod section;
pub mod text;

use crate::models::blocks::section::SectionBlock;
use crate::models::blocks::text::RichTextBlock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    RichText(RichTextBlock),
    Divider,
    Section(SectionBlock),
}
