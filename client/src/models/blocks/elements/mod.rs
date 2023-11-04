use crate::models::blocks::elements::text::LinkElement;
use crate::models::blocks::objects::text::TextBody;
use serde::{Deserialize, Serialize};
use text::RichTextSectionElement;
use crate::models::blocks::elements::emoji::EmojiElement;

pub mod text;
pub mod emoji;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BlockElement {
    RichTextSection(RichTextSectionElement),
    Text(TextBody),
    Link(LinkElement),
    Emoji(EmojiElement)
}
