extern crate core;

use async_trait::async_trait;
use mockall::automock;
use client::error::SlackClientError;
use client::message::Message;
use client::models::{Event, SocketMessage};
use client::socket_listener::SocketModeListener;
use client::SlackClient;

pub struct SlackBot {
    client: Box<dyn SlackClient + Sync>,
    listener: Box<dyn SocketModeListener>,
    plugins: Vec<Box<dyn Plugin>>
}

impl SlackBot {
    pub fn from(client: Box<dyn SlackClient + Sync>, listener: Box<dyn SocketModeListener>) -> SlackBot {
        SlackBot { client, listener, plugins: vec![] }
    }

    pub async fn run(self) -> Result<(), SlackClientError> {
        let mut listener = self.client.connect_to_socket_mode().await?;

        loop {
            let message = listener.next().await?;

            match message {
                SocketMessage::Event {
                    envelope_id,
                    payload,
                } => {
                    for plugin in &self.plugins {
                        plugin.on_event(&payload.event, &self.client);
                    }
                }
                SocketMessage::Interactive { .. } => {
                    todo!() /*Not implemented*/
                }
                SocketMessage::SlashCommand { .. } => {
                    todo!() /*Not implemented*/
                }

                SocketMessage::Hello { .. } => { /* Nothing to do */ }
                SocketMessage::Disconnect { .. } => {
                    break;
                }
            }
        }

        Ok(())
    }

    pub(crate) fn with(mut self, plugin: Box<dyn Plugin>) -> Self {
        self.plugins.push(plugin);
        self
    }
}

pub enum Action {
    DoNothing
}

#[async_trait]
#[automock]
pub trait Plugin {
    async fn on_event(&self, event: &Event, client: &Box<dyn SlackClient + Sync>) -> Action; //todo: return command pattern
}

#[cfg(test)]
mod tests {
    use std::future;
    use super::*;
    use async_trait::async_trait;
    use client::error::SlackClientError;
    use client::models::{Payload, SocketMessage};
    use client::response::ApiResponse;

    struct TestSlackClient { message: String }
    impl Default for TestSlackClient {
        fn default() -> Self {
            TestSlackClient { message: String::new() }
        }
    }
    #[async_trait]
    impl SlackClient for TestSlackClient {
        async fn message_channel(
            &self,
            channel: &str,
            message: &str,
        ) -> Result<ApiResponse, SlackClientError> {
            todo!()
        }

        async fn message_thread(
            &self,
            channel: &str,
            parent: &Message,
            message: &str,
        ) -> Result<ApiResponse, SlackClientError> {
            todo!()
        }

        async fn connect_to_socket_mode(
            &self,
        ) -> Result<Box<dyn SocketModeListener>, SlackClientError> {
            Ok(Box::new(TestSocketModeListener::default()))
        }
    }

    struct TestSocketModeListener {
        call_count: usize,
    }
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
            } else if self.call_count == 2 {
                Ok(SocketMessage::Event {
                    envelope_id: "fake-envelope-id".to_string(),
                    payload: Payload {
                        event: Event {
                            id: "fake-id".to_string(),
                            event_type: "fake-id".to_string(),
                            text: Some("this is your secret message".to_string()),
                            user: None,
                            blocks: vec![],
                            channel: None,
                            channel_type: None,
                        },
                    },
                })
            } else {
                Ok(SocketMessage::Disconnect {})
            }
        }
    }

    #[tokio::test]
    async fn disconnect_after_disconnect_message_received() {
        let bot = SlackBot::from(
            Box::new(TestSlackClient::default()),
            Box::new(TestSocketModeListener::default()),
        );

        bot.run().await.unwrap();
    }

    #[tokio::test]
    async fn forward_event_message_to_plugin() {
        let mut mock_plugin = Box::new(MockPlugin::new());
        mock_plugin.expect_on_event()
            .times(1)
            .returning(|_, _| Box::pin(future::ready(Action::DoNothing)));
        let bot = SlackBot::from(
            Box::new(TestSlackClient::default()),
            Box::new(TestSocketModeListener::default()),
        )
            .with(mock_plugin);

        bot.run().await.unwrap();
    }
}
