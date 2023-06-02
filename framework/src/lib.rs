use reqwest::{Error, Response};

pub struct SlackClient {
    bot_token: String,
}

impl SlackClient {
    pub fn new(bot_token: &str) -> SlackClient {
        SlackClient {
            bot_token: String::from(bot_token),
        }
    }

    pub async fn send_thread_reply(&self) -> Result<Response, Error> {
        let client = reqwest::Client::new();
        client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.bot_token))
            .header("User-Agent", "slackbot-client")
            .header("Accept", "application/json")
            .json(&serde_json::json!({
                "channel": "#bots",
                "thread_ts": "1685739918.465369",
                "text": "Hello world"
            }))
            .send()
            .await
    }
}
