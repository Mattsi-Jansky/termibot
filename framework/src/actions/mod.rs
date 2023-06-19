use client::models::response::Message;

pub mod handler;

pub enum Action {
    DoNothing,
    MessageChannel {
        channel: String,
        message: String,
    },
    ReplyToThread {
        channel: String,
        thread: Message,
        message: String,
    },
}
