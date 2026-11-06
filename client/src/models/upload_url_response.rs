use crate::error::SlackClientError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UploadUrlResponse {
    pub ok: bool,
    pub error: Option<String>,
    pub upload_url: Option<String>,
    pub file_id: Option<String>,
}

#[derive(Debug)]
pub struct UploadDestination {
    pub url: String,
    pub file_id: String,
}

impl UploadUrlResponse {
    pub fn into_destination(self) -> Result<UploadDestination, SlackClientError> {
        match (self.ok, self.upload_url, self.file_id) {
            (true, Some(url), Some(file_id)) => Ok(UploadDestination { url, file_id }),
            _ => Err(SlackClientError(format!(
                "Slack didn't provide an upload URL: {}",
                self.error.unwrap_or_default()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(json: &str) -> UploadUrlResponse {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn successful_response_gives_the_destination() {
        let destination = response(
            r#"{"ok": true, "upload_url": "https://files.slack.com/upload/v1/abc", "file_id": "F123"}"#,
        )
        .into_destination()
        .unwrap();

        assert_eq!(destination.url, "https://files.slack.com/upload/v1/abc");
        assert_eq!(destination.file_id, "F123");
    }

    #[test]
    fn refused_response_is_an_error_naming_the_reason() {
        let error = response(r#"{"ok": false, "error": "invalid_auth"}"#)
            .into_destination()
            .unwrap_err();

        assert!(error.0.contains("invalid_auth"), "{}", error.0);
    }

    #[test]
    fn response_without_an_upload_url_is_an_error() {
        assert!(response(r#"{"ok": true, "file_id": "F123"}"#)
            .into_destination()
            .is_err());
    }
}
