use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref STATUS_URL: Regex = Regex::new(
        r"https://([A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)+)/@[A-Za-z0-9_.-]+(?:@[A-Za-z0-9.-]+)?/(\d+)\b"
    )
    .unwrap();
}

#[derive(Debug, PartialEq)]
pub struct StatusLink {
    pub url: String,
    pub host: String,
    pub id: String,
}

impl StatusLink {
    pub fn find_in(text: &str) -> Option<StatusLink> {
        let captures = STATUS_URL.captures(text)?;
        Some(StatusLink {
            url: captures[0].to_string(),
            host: captures[1].to_string(),
            id: captures[2].to_string(),
        })
    }

    pub fn api_url(&self) -> String {
        format!("https://{}/api/v1/statuses/{}", self.host, self.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host_and_id(text: &str) -> Option<(String, String)> {
        StatusLink::find_in(text).map(|link| (link.host, link.id))
    }

    fn some(host: &str, id: &str) -> Option<(String, String)> {
        Some((host.to_string(), id.to_string()))
    }

    #[test]
    fn should_find_link_to_a_post() {
        assert_eq!(
            host_and_id("https://mastodon.social/@Mastodon/116929144390213579"),
            some("mastodon.social", "116929144390213579")
        );
    }

    #[test]
    fn should_find_link_wrapped_by_slack() {
        assert_eq!(
            host_and_id("look <https://hachyderm.io/@some_one/123|hachyderm.io/@some_one/123> wow"),
            some("hachyderm.io", "123")
        );
    }

    #[test]
    fn should_find_link_to_a_post_from_another_server() {
        assert_eq!(
            host_and_id("<https://mastodon.social/@someone@infosec.exchange/456>"),
            some("mastodon.social", "456")
        );
    }

    #[test]
    fn should_keep_the_whole_link() {
        let link = StatusLink::find_in("see <https://mas.to/@sam/789>").unwrap();

        assert_eq!(link.url, "https://mas.to/@sam/789");
    }

    #[test]
    fn should_build_api_url_for_the_post() {
        let link = StatusLink::find_in("https://mas.to/@sam/789").unwrap();

        assert_eq!(link.api_url(), "https://mas.to/api/v1/statuses/789");
    }

    #[test]
    fn should_ignore_links_that_are_not_posts() {
        for text in [
            "https://medium.com/@writer/a-title-52ed8ba68534",
            "https://www.youtube.com/@channel/videos",
            "https://www.tiktok.com/@someone/video/7234",
            "https://mastodon.social/@Mastodon",
            "https://mastodon.social/@Mastodon/123abc",
            "http://mastodon.social/@Mastodon/123",
            "no links here",
        ] {
            assert_eq!(host_and_id(text), None, "{text}");
        }
    }
}
