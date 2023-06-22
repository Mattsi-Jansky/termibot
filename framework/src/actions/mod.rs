use client::models::message_body::MessageBody;
use client::models::message_id::MessageId;

pub mod handler;

pub enum Action {
    DoNothing,
    MessageChannel {
        channel: String,
        message: MessageBody,
    },
    ReplyToThread {
        channel: String,
        thread_id: MessageId,
        message: MessageBody,
    },
}
