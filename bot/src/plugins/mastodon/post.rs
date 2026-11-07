use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Post {
    pub url: Option<String>,
    pub content: String,
    #[serde(default)]
    pub spoiler_text: String,
    #[serde(default)]
    pub sensitive: bool,
    pub account: Account,
    #[serde(default)]
    pub media_attachments: Vec<MediaAttachment>,
    pub reblog: Option<Box<Post>>,
    pub quote: Option<Quote>,
}

impl Post {
    pub fn into_original_post(self) -> Post {
        match self.reblog {
            Some(boosted) => *boosted,
            None => self,
        }
    }

    pub fn quoted_post(&self) -> Option<&Post> {
        self.quote.as_ref()?.quoted_status.as_deref()
    }
}

#[derive(Debug, Deserialize)]
pub struct Quote {
    pub quoted_status: Option<Box<Post>>,
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
    pub meta: Option<Value>,
}

impl MediaAttachment {
    pub fn duration_in_seconds(&self) -> Option<f64> {
        self.meta.as_ref()?.pointer("/original/duration")?.as_f64()
    }
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
    use super::Post;
    use serde_json::{json, Value};

    pub fn real_post() -> Post {
        serde_json::from_str(include_str!("test_data/post.json")).unwrap()
    }

    pub fn real_video_post() -> Post {
        serde_json::from_str(include_str!("test_data/video_post.json")).unwrap()
    }

    pub fn post_json() -> Value {
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

    pub const QUOTE_FALLBACK: &str = r#"<p class="quote-inline">RE: <a href="https://other.example/@alex/222">other.example/@alex/222</a></p>"#;

    pub fn quoted_post_json() -> Value {
        let mut json = post_json();
        json["url"] = json!("https://other.example/@alex/222");
        json["content"] = json!("<p>The original</p>");
        json["account"]["acct"] = json!("alex@other.example");
        json["account"]["display_name"] = json!("Alex");
        json["account"]["url"] = json!("https://other.example/@alex");
        json
    }

    pub fn post_quoting(quote: Value) -> Value {
        let mut json = post_json();
        json["content"] = json!(format!("{QUOTE_FALLBACK}<p>Look at this</p>"));
        json["quote"] = quote;
        json
    }

    pub fn post(json: Value) -> Post {
        serde_json::from_value(json).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::test_data::*;
    use super::*;
    use serde_json::json;

    #[test]
    fn should_parse_a_real_post() {
        let post = real_post();

        assert_eq!(
            post.url.as_deref(),
            Some("https://mastodon.social/@Mastodon/116929144390213579")
        );
        assert_eq!(post.account.acct, "Mastodon");
        assert_eq!(post.media_attachments[0].kind, MediaKind::Image);
        assert!(post.media_attachments[0]
            .description
            .as_deref()
            .unwrap()
            .starts_with("The Mastodon profile page for Sage"));
    }

    #[test]
    fn should_read_the_length_of_a_real_video() {
        let post = real_video_post();

        assert_eq!(post.media_attachments[0].kind, MediaKind::Video);
        assert_eq!(
            post.media_attachments[0].duration_in_seconds(),
            Some(80.662)
        );
    }

    #[test]
    fn media_without_a_length_has_no_duration() {
        let mut json = post_json();
        json["media_attachments"] = json!([
            {"type": "image", "url": "https://example.com/cat.png", "meta": {"original": {"width": 640}}},
            {"type": "video", "url": "https://example.com/cat.mp4", "meta": {"original": {"duration": "unknown"}}}
        ]);

        let post = post(json);

        assert_eq!(post.media_attachments[0].duration_in_seconds(), None);
        assert_eq!(post.media_attachments[1].duration_in_seconds(), None);
    }

    #[test]
    fn quoted_post_is_the_post_the_quote_includes() {
        let post = post(post_quoting(
            json!({"state": "accepted", "quoted_status": quoted_post_json()}),
        ));

        assert_eq!(
            post.quoted_post().unwrap().account.acct,
            "alex@other.example"
        );
    }

    #[test]
    fn quoted_post_is_unavailable_when_only_its_id_is_given() {
        let post = post(post_quoting(
            json!({"state": "accepted", "quoted_status_id": "222"}),
        ));

        assert!(post.quoted_post().is_none());
    }

    #[test]
    fn posts_nested_too_deeply_are_refused_rather_than_parsed() {
        let mut json = post_json();
        for _ in 0..100 {
            json = post_quoting(json!({"state": "accepted", "quoted_status": json}));
        }

        let result = serde_json::from_slice::<Post>(json.to_string().as_bytes());

        assert!(result.unwrap_err().to_string().contains("recursion limit"));
    }

    #[test]
    fn posts_without_a_quote_have_no_quoted_post() {
        assert!(post(post_json()).quoted_post().is_none());
    }

    #[test]
    fn should_parse_unknown_media_types() {
        let mut json = post_json();
        json["media_attachments"] = json!([{"type": "hologram", "url": null}]);

        assert_eq!(post(json).media_attachments[0].kind, MediaKind::Unknown);
    }

    #[test]
    fn original_post_of_a_boost_is_the_boosted_post() {
        let mut boosted = post_json();
        boosted["account"]["acct"] = json!("someone@other.example");
        let mut boost = post_json();
        boost["reblog"] = boosted;

        let original = post(boost).into_original_post();

        assert_eq!(original.account.acct, "someone@other.example");
    }

    #[test]
    fn original_post_of_a_regular_post_is_itself() {
        assert_eq!(post(post_json()).into_original_post().account.acct, "sam");
    }
}
