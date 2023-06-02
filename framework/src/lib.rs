use reqwest::{Error, Response};

//Example requesting all channels
// client.get("https://slack.com/api/conversations.list")
// .header("Authorization", "Bearer <bot token>")
// .header("User-Agent", "slackbot-client")
// .header("Accept", "application/json")
// .send()
// .await

pub async fn send_thread_reply() -> Result<Response, Error> {
    let client = reqwest::Client::new();
    client.post("https://slack.com/api/chat.postMessage")
        .header("Authorization", "Bearer ")
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
