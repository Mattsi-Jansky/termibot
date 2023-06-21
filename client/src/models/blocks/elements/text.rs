use builder_pattern::Builder;
use serde::{Deserialize, Serialize};
use crate::models::blocks::elements::BlockElement;

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct RichTextSectionElement {
    pub elements: Vec<BlockElement>
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct TextElement {
    pub text: String
}
