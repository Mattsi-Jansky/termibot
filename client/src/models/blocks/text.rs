use crate::models::blocks::elements::BlockElement;
use crate::models::blocks::objects::text::{Text, TextBody};
use crate::models::blocks::Block;
use builder_pattern::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct RichTextBlock {
    pub elements: Vec<BlockElement>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct HeaderBlock {
    pub text: Text,
}

impl HeaderBlock {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(text: &str) -> Block {
        Block::Header(HeaderBlock {
            text: Text::PlainText(TextBody {
                text: text.to_string(),
            }),
        })
    }
}
