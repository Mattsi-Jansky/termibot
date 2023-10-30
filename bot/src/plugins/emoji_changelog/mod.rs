use async_trait::async_trait;
use client::models::blocks::section::SectionBlock;
use client::models::blocks::text::HeaderBlock;
use client::models::message_body::MessageBody;
use client::models::socket_message::{EmojiChangedEvent, Event};
use framework::actions::Action;
use framework::dependencies::Dependencies;
use framework::plugins::Plugin;

pub struct EmojiChangelogPlugin {
    notification_channel: String,
}

impl EmojiChangelogPlugin {
    pub fn new(notification_channel: String) -> Self {
        EmojiChangelogPlugin {
            notification_channel,
        }
    }
}

#[async_trait]
impl Plugin for EmojiChangelogPlugin {
    async fn on_event(&self, event: &Event, _dependencies: &Dependencies) -> Vec<Action> {
        match event {
            Event::EmojiChanged(emoji_event) => match emoji_event {
                EmojiChangedEvent::Add(add_event) => {
                    let name = add_event.name.clone();
                    vec![Action::MessageChannel {
                        channel: self.notification_channel.clone(),
                        message: MessageBody::new(
                            vec![
                                HeaderBlock::new("Emoji changelog"),
                                SectionBlock::new_markdown(
                                    &format!(":heavy_plus_sign: :{name}: `:{name}:`")[..],
                                ),
                            ],
                            None,
                        )
                        .unwrap(),
                    }]
                }
                EmojiChangedEvent::Remove(_) => {
                    vec![]
                }
                EmojiChangedEvent::Rename(_) => {
                    vec![]
                }
            },
            _ => {
                vec![]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::models::blocks::section::SectionBlock;
    use client::models::blocks::text::HeaderBlock;
    use client::models::message_body::MessageBody;
    use client::models::message_id::MessageId;
    use client::models::socket_message::{AddEmojiEvent, EmojiChangedEvent};
    use framework::dependencies::DependenciesBuilder;

    #[tokio::test]
    async fn given_emoji_add_event_send_message_to_notification_channel() {
        let plugin = EmojiChangelogPlugin::new("#general".to_string());
        let dependencies = DependenciesBuilder::default().build();
        let event = Event::EmojiChanged(EmojiChangedEvent::Add(AddEmojiEvent {
            id: MessageId::from("testid"),
            name: "newmoji".to_string(),
        }));

        let mut result = plugin.on_event(&event, &dependencies).await;

        assert_eq!(1, result.len());
        assert_eq!(
            Action::MessageChannel {
                channel: "#general".to_string(),
                message: MessageBody::new(
                    vec![
                        HeaderBlock::new("Emoji changelog"),
                        SectionBlock::new_markdown(":heavy_plus_sign: :newmoji: `:newmoji:`")
                    ],
                    None
                )
                .unwrap(),
            },
            result.pop().unwrap()
        )
    }
}
