use builder_pattern::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct EmojiElement {
    pub name: String,
}
