use async_trait::async_trait;
use client::models::socket_message::Event;
use framework::actions::Action;
use framework::dependencies::Dependencies;
use framework::plugins::Plugin;

pub struct KarmaPlugin {
    upvote_emoji: String,
    downvote_emoji: String,
}

#[async_trait]
impl Plugin for KarmaPlugin {
    async fn on_event(&self, event: &Event, dependencies: &Dependencies) -> Vec<Action> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;


}