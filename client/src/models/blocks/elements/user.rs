use builder_pattern::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Builder)]
pub struct UserElement {
    #[serde(rename = "user_id")]
    pub id: String,
}
