use client::models::socket_message::Event;

pub(crate) fn parse_commands(event: &Event, user_id: String) -> Option<BotCommand> {
    if let Event::Message(message) = event {
        let text = message.text.clone().unwrap_or(String::new());
        if text.starts_with(format!("<@{user_id}>").as_str()) {
            let mut words: Vec<String> = text.split_whitespace().map(String::from).collect();
            words.remove(0);
            if !words.is_empty() {
                let command = words.remove(0);
                Some(BotCommand {
                    command,
                    arguments: words,
                })
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BotCommand {
    command: String,
    arguments: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_no_command_return_none() {
        let event = Event::new_test_text_message("There are more things in heaven and earth, Horatio, Than are dreamt of in your philosophy");
        let result = parse_commands(&event, "F4K3U53R1D".to_string());

        assert_eq!(result, None);
    }

    #[test]
    fn given_command_return_parsed_command() {
        let event = Event::new_test_text_message("<@F4K3U53R1D> myCommand");
        let result = parse_commands(&event, "F4K3U53R1D".to_string());

        assert_eq!(
            result,
            Some(BotCommand {
                command: "myCommand".to_string(),
                arguments: vec![]
            })
        );
    }

    #[test]
    fn given_command_with_arguments_include_arguments_in_result() {
        let event = Event::new_test_text_message("<@F4K3U53R1D> myCommand myArg1 myArg2");
        let result = parse_commands(&event, "F4K3U53R1D".to_string());

        assert_eq!(
            result,
            Some(BotCommand {
                command: "myCommand".to_string(),
                arguments: vec![String::from("myArg1"), String::from("myArg2")]
            })
        );
    }
}
