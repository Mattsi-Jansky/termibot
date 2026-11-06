mod api;
mod link;
mod mrkdwn;
mod post;
mod preview;
mod public_address;

use crate::plugins::mastodon::api::{HttpMastodonApi, MastodonApi};
use crate::plugins::mastodon::link::PostLink;
use crate::plugins::mastodon::preview::Preview;
use async_trait::async_trait;
use client::models::socket_message::{Event, MessageEvent};
use framework::actions::{Action, Image};
use framework::dependencies::Dependencies;
use framework::plugins::Plugin;
use tracing::{debug, info, warn};

pub struct MastodonPlugin;

#[async_trait]
impl Plugin for MastodonPlugin {
    async fn on_event(&self, event: &Event, dependencies: &Dependencies) -> Vec<Action> {
        let Event::Message(message) = event else {
            return vec![];
        };
        let Some(link) = message.text.as_deref().and_then(PostLink::find_in) else {
            return vec![];
        };
        let Some(http) = dependencies.get_dyn::<reqwest::Client>() else {
            warn!("No HTTP client to fetch {} with", link.url);
            return vec![];
        };

        let api = HttpMastodonApi::new(http.read().await.as_ref().clone());
        preview_replies(message, &link, &api).await
    }
}

async fn preview_replies(
    message: &MessageEvent,
    link: &PostLink,
    api: &dyn MastodonApi,
) -> Vec<Action> {
    let post = match api.fetch_post(link).await {
        Ok(post) => post.into_original_post(),
        Err(err) => {
            debug!("Not previewing {}: {}", link.url, err);
            return vec![];
        }
    };
    let preview = Preview::new(&post, link);
    let channel = message.channel.clone().unwrap_or_default();

    vec![Action::FirstSuccessful(vec![
        Action::ReplyToThreadWithImages {
            channel: channel.clone(),
            thread_id: message.id.clone(),
            message: preview.message(),
            images: download(preview.images(), api).await,
        },
        Action::ReplyToThread {
            channel,
            thread_id: message.id.clone(),
            message: preview.text_only_message(),
        },
    ])]
}

async fn download(images: Vec<Image>, api: &dyn MastodonApi) -> Vec<Image> {
    let mut downloaded = Vec::with_capacity(images.len());
    for image in images {
        match api.download_image(&image.url).await {
            Ok(content) => downloaded.push(image.with_content(content)),
            Err(err) => {
                info!("Linking to {} instead of uploading it: {}", image.url, err);
                downloaded.push(image);
            }
        }
    }
    downloaded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::mastodon::api::{FetchError, MockMastodonApi};
    use crate::plugins::mastodon::post::test_data::{post, post_json};
    use client::models::message_id::MessageId;
    use framework::dependencies::DependenciesBuilder;
    use serde_json::{json, Value};

    const IMAGE_URL: &str = "https://files.mastodon.social/media/cat.png";

    fn message_event(text: &str) -> MessageEvent {
        MessageEvent {
            id: MessageId("1700000000.000100".to_string()),
            text: Some(text.to_string()),
            user: Some("U1".to_string()),
            blocks: None,
            channel: Some("C1".to_string()),
            channel_type: Some("channel".to_string()),
        }
    }

    fn post_with_image() -> Value {
        let mut json = post_json();
        json["media_attachments"] =
            json!([{"type": "image", "url": IMAGE_URL, "description": "A cat"}]);
        json
    }

    fn link() -> PostLink {
        PostLink::find_in("https://mastodon.social/@sam/111").unwrap()
    }

    #[tokio::test]
    async fn messages_without_links_to_posts_get_no_reply() {
        let actions = MastodonPlugin
            .on_event(
                &Event::Message(message_event("https://example.com/@sam")),
                &DependenciesBuilder::default().build(),
            )
            .await;

        assert!(actions.is_empty());
    }

    #[tokio::test]
    async fn links_to_posts_get_a_preview_with_a_text_only_fallback() {
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post()
            .withf(|link| link.api_url() == "https://mastodon.social/api/v1/statuses/111")
            .returning(|_| Ok(post(post_with_image())));
        api.expect_download_image()
            .withf(|url| url == IMAGE_URL)
            .returning(|_| Ok(b"cat pixels".to_vec()));
        let event = message_event("look <https://mastodon.social/@sam/111>");

        let actions = preview_replies(&event, &link(), &api).await;

        let preview = Preview::new(&post(post_with_image()), &link());
        assert_eq!(
            actions,
            vec![Action::FirstSuccessful(vec![
                Action::ReplyToThreadWithImages {
                    channel: "C1".to_string(),
                    thread_id: event.id.clone(),
                    message: preview.message(),
                    images: vec![
                        Image::new(IMAGE_URL, "A cat").with_content(b"cat pixels".to_vec())
                    ],
                },
                Action::ReplyToThread {
                    channel: "C1".to_string(),
                    thread_id: event.id.clone(),
                    message: preview.text_only_message(),
                },
            ])]
        );
    }

    #[tokio::test]
    async fn links_to_posts_that_cannot_be_fetched_get_no_reply() {
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post()
            .returning(|_| Err(FetchError("Not a Mastodon post".to_string())));

        let actions = preview_replies(&message_event("ignored"), &link(), &api).await;

        assert!(actions.is_empty());
    }

    #[tokio::test]
    async fn images_that_cannot_be_downloaded_are_linked_instead() {
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post()
            .returning(|_| Ok(post(post_with_image())));
        api.expect_download_image()
            .returning(|_| Err(FetchError("Response is too large".to_string())));

        let actions = preview_replies(&message_event("ignored"), &link(), &api).await;

        match &actions[..] {
            [Action::FirstSuccessful(alternatives)] => match &alternatives[0] {
                Action::ReplyToThreadWithImages { images, .. } => {
                    assert_eq!(images, &vec![Image::new(IMAGE_URL, "A cat")])
                }
                other => panic!("Expected a reply with images, got {other:?}"),
            },
            other => panic!("Expected alternative replies, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn boosts_are_previewed_as_the_boosted_post() {
        let mut boosted = post_json();
        boosted["account"]["acct"] = json!("someone@other.example");
        let mut boost = post_json();
        boost["reblog"] = boosted;
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post()
            .returning(move |_| Ok(post(boost.clone())));

        let actions = preview_replies(&message_event("ignored"), &link(), &api).await;

        match &actions[..] {
            [Action::FirstSuccessful(alternatives)] => match &alternatives[1] {
                Action::ReplyToThread { message, .. } => {
                    assert_eq!(
                        message.get_text(),
                        "Mastodon post by @someone@other.example"
                    )
                }
                other => panic!("Expected a text only reply, got {other:?}"),
            },
            other => panic!("Expected alternative replies, got {other:?}"),
        }
    }
}
