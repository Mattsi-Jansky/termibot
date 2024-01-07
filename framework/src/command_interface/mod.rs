use client::models::socket_message::Event;


pub(crate) fn parse_commands(_event: &Event, _user_id: String) -> Option<BotCommand> {
    None
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
}
