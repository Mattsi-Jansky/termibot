use builder_pattern::Builder;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use crate::models::blocks::objects::text::Text;

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct SectionBlock {
    #[default(None)]
    pub text: Option<Text>,
    #[default(None)]
    pub fields: Option<Vec<Text>>
}
