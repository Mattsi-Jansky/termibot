pub mod elements;
pub mod text;

use crate::models::blocks::text::RichTextBlock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    RichText(RichTextBlock),
}
