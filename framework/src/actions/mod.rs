use client::models::message_id::MessageId;

pub mod handler;

pub enum Action {
    DoNothing,
    MessageChannel {
        channel: String,
        message: String,
    },
    ReplyToThread {
        channel: String,
        thread_id: MessageId,
        message: String,
    },
}
