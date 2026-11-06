mod fake_slack_api;

use crate::fake_slack_api::FakeSlackApi;
use client::models::blocks::section::SectionBlock;
use client::models::file_upload::FileUpload;
use client::models::message_body::MessageBody;
use client::models::message_id::MessageId;
use client::SlackClient;
use serde_json::json;
use std::collections::HashMap;

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

#[tokio::test]
async fn messages_leave_unfurling_to_slack_by_default() {
    let slack = FakeSlackApi::default().then_respond(200, MESSAGE_SENT);

    slack
        .client()
        .message_channel("C123", &MessageBody::from_text("https://example.com"))
        .await
        .unwrap();

    let body = slack.requests()[0].json();
    assert_eq!(body.get("unfurl_links"), None);
    assert_eq!(body.get("unfurl_media"), None);
}

#[tokio::test]
async fn messages_can_turn_off_unfurling() {
    let slack = FakeSlackApi::default().then_respond(200, MESSAGE_SENT);
    let message = MessageBody::from_text("https://example.com").without_unfurling();

    slack
        .client()
        .message_channel("C123", &message)
        .await
        .unwrap();

    let body = slack.requests()[0].json();
    assert_eq!(body["unfurl_links"], json!(false));
    assert_eq!(body["unfurl_media"], json!(false));
}

const UPLOAD_DESTINATION: &str = r#"{
    "ok": true,
    "upload_url": "https://files.slack.com/upload/v1/abc",
    "file_id": "F123"
}"#;
const UPLOAD_COMPLETED: &str = r#"{"ok": true, "files": [{"id": "F123"}]}"#;

fn cat_picture() -> FileUpload {
    FileUpload::new("cat.png", b"cat pixels".to_vec()).with_alt_text("A cat")
}

#[tokio::test]
async fn uploading_a_file_returns_its_id() {
    let slack = FakeSlackApi::default()
        .then_respond(200, UPLOAD_DESTINATION)
        .then_respond(200, "OK")
        .then_respond(200, UPLOAD_COMPLETED);

    let file_id = slack.client().upload_file(cat_picture()).await.unwrap();

    assert_eq!(file_id, "F123");
}

#[tokio::test]
async fn uploading_a_file_asks_for_a_destination_then_sends_content_there_then_completes() {
    let slack = FakeSlackApi::default()
        .then_respond(200, UPLOAD_DESTINATION)
        .then_respond(200, "OK")
        .then_respond(200, UPLOAD_COMPLETED);

    slack.client().upload_file(cat_picture()).await.unwrap();

    let requests = slack.requests();
    assert_eq!(
        requests[0].url,
        "https://slack.com/api/files.getUploadURLExternal"
    );
    assert_eq!(
        requests[0].form(),
        HashMap::from([
            ("filename".to_string(), "cat.png".to_string()),
            ("length".to_string(), "10".to_string()),
            ("alt_txt".to_string(), "A cat".to_string()),
        ])
    );
    assert_eq!(requests[1].url, "https://files.slack.com/upload/v1/abc");
    assert_eq!(requests[1].body, b"cat pixels");
    assert_eq!(
        requests[2].url,
        "https://slack.com/api/files.completeUploadExternal"
    );
    let files: serde_json::Value = serde_json::from_str(&requests[2].form()["files"]).unwrap();
    assert_eq!(files, json!([{"id": "F123"}]));
}

#[tokio::test]
async fn upload_fails_when_slack_refuses_a_destination() {
    let slack =
        FakeSlackApi::default().then_respond(200, r#"{"ok": false, "error": "invalid_auth"}"#);

    let result = slack.client().upload_file(cat_picture()).await;

    assert!(result.unwrap_err().0.contains("invalid_auth"));
    assert_eq!(slack.requests().len(), 1);
}

#[tokio::test]
async fn upload_fails_without_completing_when_sending_content_fails() {
    let slack = FakeSlackApi::default()
        .then_respond(200, UPLOAD_DESTINATION)
        .then_respond(500, "Internal Server Error");

    let result = slack.client().upload_file(cat_picture()).await;

    assert!(result.is_err());
    assert_eq!(slack.requests().len(), 2);
}

#[tokio::test]
async fn upload_fails_when_completing_it_fails() {
    let slack = FakeSlackApi::default()
        .then_respond(200, UPLOAD_DESTINATION)
        .then_respond(200, "OK")
        .then_respond(200, r#"{"ok": false, "error": "file_not_found"}"#);

    let result = slack.client().upload_file(cat_picture()).await;

    assert!(result.unwrap_err().0.contains("file_not_found"));
}
