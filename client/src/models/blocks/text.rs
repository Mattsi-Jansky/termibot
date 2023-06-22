use crate::models::blocks::elements::BlockElement;
use builder_pattern::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct RichTextBlock {
    pub elements: Vec<BlockElement>,
}
