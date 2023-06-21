use serde::{Deserialize, Serialize};
use text::{RichTextSectionElement, TextElement};

pub mod text;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockElement {
    RichTextSection(RichTextSectionElement),
    Text(TextElement),
}
