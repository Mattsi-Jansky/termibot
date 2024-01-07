use rvstruct::ValueStruct;
use serde::{Deserialize, Serialize};

/// Corresponds to response of auth.test method: https://api.slack.com/methods/auth.test
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct UserDetailsMessage {
    pub user_id: String
}
