mod fake_slack_api;

use crate::fake_slack_api::FakeSlackApi;
use client::models::blocks::section::SectionBlock;
use client::models::message_body::MessageBody;
use client::models::message_id::MessageId;
use client::SlackClient;
use serde_json::json;

const MESSAGE_SENT: &str = r#"{"ok": true}"#;

#[tokio::test]
async fn thread_replies_include_blocks() {
    let slack = FakeSlackApi::default().then_respond(200, MESSAGE_SENT);
    let message = MessageBody::new(
        vec![SectionBlock::new_markdown("*hello*")],
        Some("hello".to_string()),
    )
    .unwrap();

    slack
        .client()
        .message_thread(
            "C123",
            &MessageId::new("1700000000.000100".to_string()),
            &message,
        )
        .await
        .unwrap();

    let requests = slack.requests();
    assert_eq!(requests[0].url, "https://slack.com/api/chat.postMessage");
    assert_eq!(
        requests[0].json(),
        json!({
            "channel": "C123",
            "thread_ts": "1700000000.000100",
            "text": "hello",
            "blocks": [{"type": "section", "text": {"type": "mrkdwn", "text": "*hello*"}}]
        })
    );
}
