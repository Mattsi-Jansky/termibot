use crate::models::blocks::elements::BlockElement;
use builder_pattern::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use crate::models::blocks::objects::text::Text;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct RichTextBlock {
    pub elements: Vec<BlockElement>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct HeaderBlock {
    pub text: Text
}
