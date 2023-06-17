pub mod handler;

#[derive(Clone)]
pub enum Action {
    DoNothing,
    MessageChannel{channel: String, message: String }
}
