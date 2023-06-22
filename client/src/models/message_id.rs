use rvstruct::ValueStruct;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, ValueStruct, Clone)]
pub struct MessageId(pub String);
