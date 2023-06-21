pub mod elements;
pub mod text;

use serde::{Deserialize, Serialize};
use crate::models::blocks::text::RichTextBlock;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    RichText(RichTextBlock)
}
