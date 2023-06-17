use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Message {
    #[serde(rename = "ts")]
    pub id: String,
    pub text: String,
    pub user: String,
}
