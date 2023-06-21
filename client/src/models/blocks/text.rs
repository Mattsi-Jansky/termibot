use crate::models::blocks::elements::BlockElement;
use builder_pattern::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct RichTextBlock {
    pub elements: Vec<BlockElement>,
}
