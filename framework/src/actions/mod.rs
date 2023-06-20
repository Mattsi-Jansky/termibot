use client::models::http_response::Message;

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
