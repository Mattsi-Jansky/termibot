use client::error::SlackClientError;
use client::message::Message;
use client::models::SocketMessage;
use client::SlackClient;
use client::socket_listener::SocketModeListener;

pub struct SlackBot {
    client: Box<dyn SlackClient>,
    listener: Box<dyn SocketModeListener>,
}

impl SlackBot {
    pub fn from(client: Box<dyn SlackClient>,
                listener: Box<dyn SocketModeListener>) -> SlackBot {
        SlackBot {client, listener}
    }

    pub async fn run(self) -> Result<(), SlackClientError> {
        let mut listener = self.client.connect_to_socket_mode().await?;

        loop {
            let message = listener.next().await?;

            match message {
                SocketMessage::Event {
                    envelope_id,
                    payload: _,
                } => {
                    todo!()
                }
                SocketMessage::Interactive { .. } => { todo!()/*Not implemented*/ }
                SocketMessage::SlashCommand { .. } => { todo!()/*Not implemented*/ }

                SocketMessage::Hello { .. } => { /* Nothing to do */ }
                SocketMessage::Disconnect { .. } => {
                    break;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use client::error::SlackClientError;
    use client::models::SocketMessage;
    use client::response::ApiResponse;
    use super::*;

    struct TestSlackClient {}
    impl Default for TestSlackClient {
        fn default() -> Self {
            TestSlackClient {}
        }
    }
    #[async_trait]
    impl SlackClient for TestSlackClient {
        async fn message_channel(&self, channel: &str, message: &str) -> Result<ApiResponse, SlackClientError> {
            todo!()
        }

        async fn message_thread(&self, channel: &str, parent: &Message, message: &str) -> Result<ApiResponse, SlackClientError> {
            todo!()
        }

        async fn connect_to_socket_mode(&self) -> Result<Box<dyn SocketModeListener>, SlackClientError> {
            Ok(Box::new(TestSocketModeListener::default()))
        }
    }

    struct TestSocketModeListener { call_count: usize }
    impl Default for TestSocketModeListener {
        fn default() -> Self {
            TestSocketModeListener { call_count: 0 }
        }
    }
    #[async_trait]
    impl SocketModeListener for TestSocketModeListener {
        async fn next(&mut self) -> serde_json::error::Result<SocketMessage> {
            self.call_count += 1;

            if self.call_count == 1 {
                Ok(SocketMessage::Hello {})
            }
            else {
                Ok(SocketMessage::Disconnect {})
            }
        }
    }

    #[tokio::test]
    async fn disconnect_after_disconnect_message_received() {
        let bot = SlackBot::from(
            Box::new(TestSlackClient::default()),
            Box::new(TestSocketModeListener::default())
        );

        bot.run().await.unwrap();
    }
}
