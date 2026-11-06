use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Status {
    pub url: Option<String>,
    pub content: String,
    #[serde(default)]
    pub spoiler_text: String,
    #[serde(default)]
    pub sensitive: bool,
    pub account: Account,
    #[serde(default)]
    pub media_attachments: Vec<MediaAttachment>,
    pub reblog: Option<Box<Status>>,
}

impl Status {
    pub fn into_original_post(self) -> Status {
        match self.reblog {
            Some(boosted) => *boosted,
            None => self,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Account {
    pub acct: String,
    pub display_name: String,
    pub url: String,
    pub avatar_static: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MediaAttachment {
    #[serde(rename = "type")]
    pub kind: MediaKind,
    pub url: Option<String>,
    pub preview_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Gifv,
    Video,
    Audio,
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
pub mod test_data {
    use super::Status;
    use serde_json::{json, Value};

    pub fn real_status() -> Status {
        serde_json::from_str(include_str!("test_data/status.json")).unwrap()
    }

    pub fn status_json() -> Value {
        json!({
            "url": "https://mastodon.social/@sam/111",
            "content": "<p>Hello</p>",
            "spoiler_text": "",
            "sensitive": false,
            "account": {
                "acct": "sam",
                "display_name": "Sam",
                "url": "https://mastodon.social/@sam",
                "avatar_static": "https://files.mastodon.social/avatars/sam.png"
            },
            "media_attachments": [],
            "reblog": null
        })
    }

    pub fn status(json: Value) -> Status {
        serde_json::from_value(json).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::test_data::*;
    use super::*;
    use serde_json::json;

    #[test]
    fn should_parse_a_real_status() {
        let status = real_status();

        assert_eq!(
            status.url.as_deref(),
            Some("https://mastodon.social/@Mastodon/116929144390213579")
        );
        assert_eq!(status.account.acct, "Mastodon");
        assert_eq!(status.media_attachments[0].kind, MediaKind::Image);
        assert!(status.media_attachments[0]
            .description
            .as_deref()
            .unwrap()
            .starts_with("The Mastodon profile page for Sage"));
    }

    #[test]
    fn should_parse_unknown_media_types() {
        let mut json = status_json();
        json["media_attachments"] = json!([{"type": "hologram", "url": null}]);

        assert_eq!(status(json).media_attachments[0].kind, MediaKind::Unknown);
    }

    #[test]
    fn original_post_of_a_boost_is_the_boosted_post() {
        let mut boosted = status_json();
        boosted["account"]["acct"] = json!("someone@other.example");
        let mut boost = status_json();
        boost["reblog"] = boosted;

        let original = status(boost).into_original_post();

        assert_eq!(original.account.acct, "someone@other.example");
    }

    #[test]
    fn original_post_of_a_regular_post_is_itself() {
        assert_eq!(
            status(status_json()).into_original_post().account.acct,
            "sam"
        );
    }
}
