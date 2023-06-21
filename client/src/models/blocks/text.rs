use builder_pattern::Builder;
use serde::{Deserialize, Serialize};
use crate::models::blocks::elements::BlockElement;

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct RichTextBlock {
    pub elements: Vec<BlockElement>
}
