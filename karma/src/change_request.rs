pub struct ChangeRequest {
    pub name: String,
    pub amount: i64,
}

impl ChangeRequest {
    pub(crate) fn new(name: &str, amount: i64) -> Self {
        Self {
            name: name.to_string(),
            amount,
        }
    }
}
