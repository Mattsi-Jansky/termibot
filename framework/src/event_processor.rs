use crate::enriched_event::{CommandData, EnrichedEvent};
use client::models::socket_message::{Event, MessageEvent};

pub struct EventProcessor {
    bot_name: String,
    bot_id: String,
}

impl EventProcessor {
    pub fn new(bot_name: String, bot_id: String) -> Self {
        Self { bot_name, bot_id }
    }

    /// Process a raw Event into an EnrichedEvent if the bot is being addressed
    pub fn process(&self, event: &Event) -> Option<EnrichedEvent> {
        match event {
            Event::Message(msg_event) => self.process_message(msg_event),
            _ => None,
        }
    }

    fn process_message(&self, msg_event: &MessageEvent) -> Option<EnrichedEvent> {
        let text = msg_event.text.as_ref()?;
        let trimmed = text.trim();

        let (is_addressed, remaining_text) = self.is_bot_addressed(trimmed);
        let parts: Vec<&str> = remaining_text.split_whitespace().collect();
        if !is_addressed || parts.is_empty() {
            None
        } else {
            let command = parts[0].to_lowercase();
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            let raw_args = if parts.len() > 1 {
                parts[1..].join(" ")
            } else {
                String::new()
            };
            let channel = msg_event.channel.clone().unwrap_or_else(|| {
                // For DMs, channel might be in a different field
                msg_event.channel_type.clone().unwrap_or_default()
            });
            let user = msg_event.user.clone().unwrap_or_default();

            Some(EnrichedEvent::Command(CommandData {
                command,
                args,
                raw_args,
                channel,
                user
            }))
        }
    }

    fn is_bot_addressed<'a>(&self, text: &'a str) -> (bool, &'a str) {
        // Check for @bot_id mention (e.g., <@U123456>)
        let mention_prefix = format!("<@{}>", self.bot_id);
        if let Some(stripped) = text.strip_prefix(&mention_prefix) {
            return (true, stripped.trim());
        }

        // Check for @bot_name mention
        let at_mention = format!("@{}", self.bot_name);
        if let Some(stripped) = text.strip_prefix(&at_mention) {
            return (true, stripped.trim());
        }

        // Check for plain bot name at the start
        let name_lower = self.bot_name.to_lowercase();
        let text_lower = text.to_lowercase();

        // Check variations like "bot_name:" or "bot_name,"
        for suffix in &[":", ",", " "] {
            let pattern = format!("{}{}", name_lower, suffix);
            if text_lower.starts_with(&pattern) {
                let remaining = &text[pattern.len()..];
                return (true, remaining.trim());
            }
        }

        (false, text)
    }
}

#[cfg(test)]
mod tests {
    use client::models::message_id::MessageId;
    use super::*;

    fn create_test_processor() -> EventProcessor {
        EventProcessor::new("testbot".to_string(), "U123456".to_string())
    }

    fn create_test_message(text: &str) -> MessageEvent {
        MessageEvent {
            id: MessageId("test_id".to_string()),
            text: Some(text.to_string()),
            user: Some("U789".to_string()),
            blocks: Some(vec![]),
            channel: Some("#general".to_string()),
            channel_type: Some("channel".to_string()),
        }
    }

    #[test]
    fn should_process_direct_mention() {
        let processor = create_test_processor();
        let msg = create_test_message("<@U123456> hello world");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_some());

        if let Some(EnrichedEvent::Command(cmd)) = result {
            assert_eq!(cmd.command, "hello");
            assert_eq!(cmd.args, vec!["world"]);
            assert_eq!(cmd.raw_args, "world");
        } else {
            panic!("Expected Command variant");
        }
    }

    #[test]
    fn should_process_at_name_mention() {
        let processor = create_test_processor();
        let msg = create_test_message("@testbot deploy production");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_some());

        if let Some(EnrichedEvent::Command(cmd)) = result {
            assert_eq!(cmd.command, "deploy");
            assert_eq!(cmd.args, vec!["production"]);
        } else {
            panic!("Expected Command variant");
        }
    }

    #[test]
    fn should_process_name_with_colon() {
        let processor = create_test_processor();
        let msg = create_test_message("testbot: status check");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_some());

        if let Some(EnrichedEvent::Command(cmd)) = result {
            assert_eq!(cmd.command, "status");
            assert_eq!(cmd.args, vec!["check"]);
        } else {
            panic!("Expected Command variant");
        }
    }

    #[test]
    fn should_ignore_non_addressed_message() {
        let processor = create_test_processor();
        let msg = create_test_message("just a regular message");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_none());
    }

    #[test]
    fn should_parse_command_with_multiple_args() {
        let processor = create_test_processor();
        let msg = create_test_message("@testbot remind me to check the logs tomorrow");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_some());

        if let Some(EnrichedEvent::Command(cmd)) = result {
            assert_eq!(cmd.command, "remind");
            assert_eq!(cmd.args, vec!["me", "to", "check", "the", "logs", "tomorrow"]);
            assert_eq!(cmd.raw_args, "me to check the logs tomorrow");
        } else {
            panic!("Expected Command variant");
        }
    }

    #[test]
    fn should_ignore_mid_message_mention() {
        let processor = create_test_processor();
        let msg = create_test_message("hey @testbot how are you");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_none());
    }

    #[test]
    fn should_parse_command_with_no_args() {
        let processor = create_test_processor();
        let msg = create_test_message("@testbot help");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_some());

        if let Some(EnrichedEvent::Command(cmd)) = result {
            assert_eq!(cmd.command, "help");
            assert!(cmd.args.is_empty());
            assert_eq!(cmd.raw_args, "");
        } else {
            panic!("Expected Command variant");
        }
    }

    #[test]
    fn should_ignore_empty_message_after_mention() {
        let processor = create_test_processor();
        let msg = create_test_message("@testbot ");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_none());
    }

    #[test]
    fn should_parse_case_insensitive_command() {
        let processor = create_test_processor();
        let msg = create_test_message("@testbot DEPLOY Production");
        let event = Event::Message(msg);

        let result = processor.process(&event);
        assert!(result.is_some());

        if let Some(EnrichedEvent::Command(cmd)) = result {
            assert_eq!(cmd.command, "deploy"); // Should be lowercase
            assert_eq!(cmd.args, vec!["Production"]); // Args preserve case
        } else {
            panic!("Expected Command variant");
        }
    }
}