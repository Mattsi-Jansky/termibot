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
    pub authorizations: Vec<Authorization>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Authorization {
    pub user_id: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Message(MessageEvent),
    EmojiChanged(EmojiChangedEvent),

    //-- Unimplemented event types
    AppHomeOpened,
    AppMention,
    AppRateLimited,
    AppRequested,
    AppUninstalled,
    CallRejected,
    ChannelArchive,
    ChannelCreated,
    ChannelDeleted,
    ChannelHistoryChanged,
    ChannelIdChanged,
    ChannelLeft,
    ChannelRename,
    ChannelShared,
    ChannelUnarchive,
    ChannelUnshared,
    DndUpdated,
    DndUpdatedUser,
    EmailDomainChanged,
    FileChange,
    FileCommentAdded,
    FileCommentDeleted,
    FileCommentEdited,
    FileCreated,
    FileDeleted,
    FilePublic,
    FileShared,
    FileUnshared,
    GridMigrationFinished,
    GridMigrationStarted,
    GroupArchive,
    GroupClose,
    GroupDeleted,
    GroupHistoryChanged,
    GroupLeft,
    GroupOpen,
    GroupRename,
    GroupUnarchive,
    ImClose,
    ImCreated,
    ImHistoryChanged,
    ImOpen,
    InviteRequested,
    LinkShared,
    MemberJoinedChannel,
    MemberLeftChannel,
    // Of course there are 5 that randomly break the naming convention, thanks Slack
    #[serde(rename = "message.app_home")]
    MessageAppHome,
    #[serde(rename = "message.channels")]
    MessageChannels,
    #[serde(rename = "message.groups")]
    MessageGroups,
    #[serde(rename = "message.im")]
    MessageIm,
    #[serde(rename = "message.mpim")]
    MessageMpim,
    MessageMetadataDeleted,
    MessageMetadataPosted,
    MessageMetadataUpdated,
    PinAdded,
    PinRemoved,
    ReactionAdded,
    ReactionRemoved,
    ResourcesAdded,
    ResourcesRemoved,
    ScopeDenied,
    ScopeGranted,
    SharedChannelInviteAccepted,
    SharedChannelInviteApproved,
    SharedChannelInviteDeclined,
    SharedChannelInviteReceived,
    StarAdded,
    StarRemoved,
    SubteamCreated,
    SubteamMembersChanged,
    SubteamSelfAdded,
    SubteamSelfRemoved,
    SubteamUpdated,
    TeamAccessGranted,
    TeamAccessRevoked,
    TeamDomainChange,
    TeamJoin,
    TeamRename,
    TokensRevoked,
    UrlVerification,
    UserChange,
    UserHuddleChanged,
    UserProfileChanged,
    UserResourceDenied,
    UserResourceGranted,
    UserResourceRemoved,
    UserStatusChanged,
    WorkflowDeleted,
    WorkflowPublished,
    WorkflowStepDeleted,
    WorkflowStepExecute,
    WorkflowUnpublished,
}

impl Event {
    pub fn new_test_text_message(message: &str) -> Event {
        Event::Message(MessageEvent {
            id: MessageId("myMessageId".to_string()),
            text: Some(message.to_string()),
            user: None,
            blocks: None,
            channel: None,
            channel_type: None,
        })
    }
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct MessageEvent {
    #[serde(rename = "event_ts")]
    pub id: MessageId,
    pub text: Option<String>,
    pub user: Option<String>,
    pub blocks: Option<Vec<Block>>,
    pub channel: Option<String>,
    pub channel_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "subtype", rename_all = "snake_case")]
pub enum EmojiChangedEvent {
    Add(AddEmojiEvent),
    Remove(RemoveEmojiEvent),
    Rename(RenameEmojiEvent),
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct AddEmojiEvent {
    #[serde(rename = "event_ts")]
    pub id: MessageId,
    pub name: String,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct RemoveEmojiEvent {
    #[serde(rename = "event_ts")]
    pub id: MessageId,
    pub names: Vec<String>,
}

#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct RenameEmojiEvent {
    #[serde(rename = "event_ts")]
    pub id: MessageId,
    pub old_name: String,
    pub new_name: String,
}

pub enum MaybeRelevantSocketMessage {
    Relevant(serde_json::error::Result<SocketMessage>),
    Irrelevant,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::blocks::elements::text::RichTextSectionElement;
    use crate::models::blocks::elements::BlockElement;
    use crate::models::blocks::objects::text::TextBody;
    use crate::models::blocks::text::RichTextBlock;

    const FAKE_NEW_MESSAGE_EVENT: &str = "{ \"client_msg_id\": \"aa022dae-607c-4e24-b0e1-f96c08855f4f\", \"type\": \"message\", \"text\": \"wat\", \"user\": \"U118BF6LQ\", \"ts\": \"1687458843.576569\", \"blocks\": [ { \"type\": \"rich_text\", \"block_id\": \"ZrfB\", \"elements\": [ { \"type\": \"rich_text_section\", \"elements\": [ { \"type\": \"text\", \"text\": \"wat\" } ] } ] } ], \"team\": \"T0G5PM4NR\", \"channel\": \"DEAS25LNP\", \"event_ts\": \"1687458843.576569\", \"channel_type\": \"im\"}";
    const FAKE_NEW_EMOJI_EVENT: &str = "{ \"type\": \"emoji_changed\", \"subtype\": \"add\", \"name\": \"blobcat_knife\", \"value\": \"https://emoji.slack-edge.com/T0G5PM4NR/blobcat_knife/8ce3359f5936936a.png\", \"event_ts\": \"1687458875.040100\"}";
    const FAKE_REMOVED_EMOJI_EVENT: &str = "{	\"type\": \"emoji_changed\",	\"subtype\": \"remove\",	\"names\": [\"picard_facepalm\"],	\"event_ts\" : \"1361482916.000004\"}";
    const FAKE_RENAMED_EMOJI_EVENT: &str = "{	\"type\": \"emoji_changed\",	\"subtype\": \"rename\",	\"old_name\": \"grin\",	\"new_name\": \"cheese-grin\",	\"value\": \"https://my.slack.com/emoji/picard_facepalm/db8e287430eaa459.gif\",	\"event_ts\" : \"1361482916.000004\"}";

    #[test]
    fn should_parse_message_event() {
        let result: Event = serde_json::from_str(FAKE_NEW_MESSAGE_EVENT).unwrap();

        if let Event::Message(message) = result {
            assert_eq!(message.id, "1687458843.576569".into());
            assert_eq!(message.text.unwrap(), "wat".to_string());
            assert_eq!(
                message.blocks.unwrap(),
                vec![Block::RichText(
                    RichTextBlock::new()
                        .elements(vec![BlockElement::RichTextSection(
                            RichTextSectionElement::new()
                                .elements(vec![BlockElement::Text(
                                    TextBody::new().text("wat".to_string()).build()
                                )])
                                .build()
                        )])
                        .build()
                )]
            );
        } else {
            panic!("Wrong type of event")
        }
    }

    #[test]
    fn should_parse_new_emoji_event() {
        let result: Event = serde_json::from_str(FAKE_NEW_EMOJI_EVENT).unwrap();

        if let Event::EmojiChanged(EmojiChangedEvent::Add(result)) = result {
            assert_eq!(result.id, "1687458875.040100".into());
            assert_eq!(result.name, "blobcat_knife".to_string());
        } else {
            panic!("Wrong type of event")
        }
    }

    #[test]
    fn should_parse_removed_emoji_event() {
        let result: Event = serde_json::from_str(FAKE_REMOVED_EMOJI_EVENT).unwrap();

        if let Event::EmojiChanged(EmojiChangedEvent::Remove(result)) = result {
            assert_eq!(result.id, "1361482916.000004".into());
            assert_eq!(result.names, vec!["picard_facepalm".to_string()]);
        } else {
            panic!("Wrong type of event")
        }
    }

    #[test]
    fn should_parse_renamed_emoji_event() {
        let result: Event = serde_json::from_str(FAKE_RENAMED_EMOJI_EVENT).unwrap();

        if let Event::EmojiChanged(EmojiChangedEvent::Rename(result)) = result {
            assert_eq!(result.id, "1361482916.000004".into());
            assert_eq!(result.old_name, "grin".to_string());
            assert_eq!(result.new_name, "cheese-grin".to_string())
        }
    }
}
