use crate::models::blocks::objects::text::TextBody;
use serde::{Deserialize, Serialize};
use text::RichTextSectionElement;

pub mod text;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockElement {
    RichTextSection(RichTextSectionElement),
    Text(TextBody),
}
