use crate::models::blocks::Block;
use crate::models::message_id::MessageId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum SocketMessage {
    #[serde(rename = "hello")]
    Hello {},
    #[serde(rename = "disconnect")]
    Disconnect {},
    #[serde(rename = "events_api")]
    Event {
        envelope_id: String,
        payload: Payload,
    },
    #[serde(rename = "interactive")]
    Interactive { envelope_id: String },
    #[serde(rename = "slash_commands")]
    SlashCommand { envelope_id: String },
}

// Ignores the type field, because it seems to always be `event_callback`
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Payload {
    pub event: Event,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Message(MessageEvent),
    EmojiChanged(EmojiChangedEvent)
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct MessageEvent {
    #[serde(rename = "event_ts")]
    pub id: MessageId,
    pub text: Option<String>,
    pub user: Option<String>,
    pub blocks: Vec<Block>,
    pub channel: Option<String>,
    pub channel_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum EmojiChangedEvent {
    Add(AddEmojiEvent),
    Remove,
    Rename
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct AddEmojiEvent {
    #[serde(rename = "event_ts")]
    pub id: MessageId,
    name: String
}

#[cfg(test)]
mod tests {
    use crate::models::blocks::elements::BlockElement;
    use crate::models::blocks::elements::text::RichTextSectionElement;
    use crate::models::blocks::objects::text::TextBody;
    use crate::models::blocks::text::RichTextBlock;
    use super::*;

    const FAKE_NEW_MESSAGE_EVENT: &str = "{ \"client_msg_id\": \"aa022dae-607c-4e24-b0e1-f96c08855f4f\", \"type\": \"message\", \"text\": \"wat\", \"user\": \"U118BF6LQ\", \"ts\": \"1687458843.576569\", \"blocks\": [ { \"type\": \"rich_text\", \"block_id\": \"ZrfB\", \"elements\": [ { \"type\": \"rich_text_section\", \"elements\": [ { \"type\": \"text\", \"text\": \"wat\" } ] } ] } ], \"team\": \"T0G5PM4NR\", \"channel\": \"DEAS25LNP\", \"event_ts\": \"1687458843.576569\", \"channel_type\": \"im\"}";
    const FAKE_NEW_EMOJI_EVENT: &str = "{ \"type\": \"emoji_changed\", \"subtype\": \"add\", \"name\": \"blobcat_knife\", \"value\": \"https://emoji.slack-edge.com/T0G5PM4NR/blobcat_knife/8ce3359f5936936a.png\", \"event_ts\": \"1687458875.040100\"}";

    #[test]
    fn should_parse_message_event() {
        let result: Event = serde_json::from_str(FAKE_NEW_MESSAGE_EVENT).unwrap();

        if let Event::Message(message) = result {
            assert_eq!(message.id, "1687458843.576569".into());
            assert_eq!(message.text.unwrap(), "wat".to_string());
            assert_eq!(message.blocks, vec![Block::RichText(RichTextBlock::new()
                .elements(vec![BlockElement::RichTextSection(
                    RichTextSectionElement::new()
                        .elements(vec![BlockElement::Text(
                            TextBody::new().text("wat".to_string()).build()
                        )])
                        .build()
                )])
                .build())]);
        } else { panic!("Wrong type of event") }
    }

    #[test]
    fn should_parse_new_emoji_event() {
        let result: Event = serde_json::from_str(FAKE_NEW_EMOJI_EVENT).unwrap();

        if let Event::EmojiChanged(result) = result {
            if let EmojiChangedEvent::Add(result) = result {
                assert_eq!(result.id, "1687458875.040100".into());
                assert_eq!(result.name, "blobcat_knife".to_string());
            } else { panic!("Wrong type of event") }
        } else { panic!("Wrong type of event") }
    }
}
