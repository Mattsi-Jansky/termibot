use builder_pattern::Builder;
use serde::{Deserialize, Serialize};
use crate::models::blocks::objects::text::Text;

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct SectionBlock {
    text: Option<Text>,
    fields: Vec<Text>
}
