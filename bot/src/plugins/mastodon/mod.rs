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
        preview_messages(message, &link, &api).await
    }
}

async fn preview_messages(
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
    let preview_of_post = preview_message(message, &Preview::new(&post, link), api).await;

    match post.quoted_post() {
        Some(quoted) => {
            let preview_of_quoted_post =
                preview_message(message, &Preview::for_quoted_post(quoted, link), api).await;
            vec![Action::InOrder(vec![
                preview_of_post,
                preview_of_quoted_post,
            ])]
        }
        None => vec![preview_of_post],
    }
}

async fn preview_message(
    message: &MessageEvent,
    preview: &Preview,
    api: &dyn MastodonApi,
) -> Action {
    let linked_images = preview.images();
    let uploadable_images = download(linked_images.clone(), api).await;

    let mut alternatives = vec![];
    if uploadable_images
        .iter()
        .any(|image| image.content.is_some())
    {
        alternatives.push(preview_with_images(message, preview, uploadable_images));
    }
    alternatives.push(preview_with_images(message, preview, linked_images));
    alternatives.push(text_only_preview(message, preview));

    Action::FirstSuccessful(alternatives)
}

fn preview_with_images(message: &MessageEvent, preview: &Preview, images: Vec<Image>) -> Action {
    Action::MessageWithImages {
        channel: message.channel.clone().unwrap_or_default(),
        thread_id: message.thread_ts.clone(),
        message: preview.message(),
        images,
    }
}

fn text_only_preview(message: &MessageEvent, preview: &Preview) -> Action {
    let channel = message.channel.clone().unwrap_or_default();
    let message_body = preview.text_only_message();
    match &message.thread_ts {
        Some(thread_id) => Action::ReplyToThread {
            channel,
            thread_id: thread_id.clone(),
            message: message_body,
        },
        None => Action::MessageChannel {
            channel,
            message: message_body,
        },
    }
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
    use crate::plugins::mastodon::post::test_data::{
        post, post_json, post_quoting, quoted_post_json,
    };
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
            thread_ts: None,
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
    async fn messages_without_links_to_posts_get_no_preview() {
        let actions = MastodonPlugin
            .on_event(
                &Event::Message(message_event("https://example.com/@sam")),
                &DependenciesBuilder::default().build(),
            )
            .await;

        assert!(actions.is_empty());
    }

    #[tokio::test]
    async fn links_to_posts_get_a_preview_falling_back_to_linked_images_then_text_only() {
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post()
            .withf(|link| link.api_url() == "https://mastodon.social/api/v1/statuses/111")
            .returning(|_| Ok(post(post_with_image())));
        api.expect_download_image()
            .withf(|url| url == IMAGE_URL)
            .returning(|_| Ok(b"cat pixels".to_vec()));
        let event = message_event("look <https://mastodon.social/@sam/111>");

        let actions = preview_messages(&event, &link(), &api).await;

        let preview = Preview::new(&post(post_with_image()), &link());
        assert_eq!(
            actions,
            vec![Action::FirstSuccessful(vec![
                preview_with_images(
                    &event,
                    &preview,
                    vec![Image::new(IMAGE_URL, "A cat").with_content(b"cat pixels".to_vec())]
                ),
                preview_with_images(&event, &preview, vec![Image::new(IMAGE_URL, "A cat")]),
                text_only_preview(&event, &preview),
            ])]
        );
    }

    #[test]
    fn previews_of_links_posted_in_the_channel_go_to_the_channel() {
        let event = message_event("look <https://mastodon.social/@sam/111>");
        let preview = Preview::new(&post(post_with_image()), &link());

        assert!(matches!(
            preview_with_images(&event, &preview, vec![]),
            Action::MessageWithImages { channel, thread_id: None, .. } if channel == "C1"
        ));
        assert!(matches!(
            text_only_preview(&event, &preview),
            Action::MessageChannel { channel, .. } if channel == "C1"
        ));
    }

    #[test]
    fn previews_of_links_posted_in_a_thread_go_to_that_thread() {
        let mut event = message_event("look <https://mastodon.social/@sam/111>");
        let thread = MessageId("1699999999.000001".to_string());
        event.thread_ts = Some(thread.clone());
        let preview = Preview::new(&post(post_with_image()), &link());

        assert!(matches!(
            preview_with_images(&event, &preview, vec![]),
            Action::MessageWithImages { channel, thread_id: Some(thread_id), .. }
                if channel == "C1" && thread_id == thread
        ));
        assert!(matches!(
            text_only_preview(&event, &preview),
            Action::ReplyToThread { channel, thread_id, .. }
                if channel == "C1" && thread_id == thread
        ));
    }

    #[tokio::test]
    async fn posts_quoting_another_post_get_a_preview_of_it_afterwards() {
        let quote_post = || {
            post(post_quoting(
                json!({"state": "accepted", "quoted_status": quoted_post_json()}),
            ))
        };
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post().returning(move |_| Ok(quote_post()));
        let event = message_event("look <https://mastodon.social/@sam/111>");

        let actions = preview_messages(&event, &link(), &api).await;

        let post = quote_post();
        let preview = Preview::new(&post, &link());
        let quoted_preview = Preview::for_quoted_post(post.quoted_post().unwrap(), &link());
        assert_eq!(
            actions,
            vec![Action::InOrder(vec![
                Action::FirstSuccessful(vec![
                    preview_with_images(&event, &preview, vec![]),
                    text_only_preview(&event, &preview),
                ]),
                Action::FirstSuccessful(vec![
                    preview_with_images(&event, &quoted_preview, vec![]),
                    text_only_preview(&event, &quoted_preview),
                ]),
            ])]
        );
    }

    #[tokio::test]
    async fn posts_quoting_a_post_that_cannot_be_shown_get_a_single_preview() {
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post().returning(|_| {
            Ok(post(post_quoting(
                json!({"state": "pending", "quoted_status": null}),
            )))
        });

        let actions = preview_messages(&message_event("ignored"), &link(), &api).await;

        assert!(matches!(&actions[..], [Action::FirstSuccessful(_)]));
    }

    #[tokio::test]
    async fn links_to_posts_that_cannot_be_fetched_get_no_preview() {
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post()
            .returning(|_| Err(FetchError("Not a Mastodon post".to_string())));

        let actions = preview_messages(&message_event("ignored"), &link(), &api).await;

        assert!(actions.is_empty());
    }

    #[tokio::test]
    async fn when_no_images_can_be_downloaded_the_preview_starts_with_linked_images() {
        let mut api = MockMastodonApi::new();
        api.expect_fetch_post()
            .returning(|_| Ok(post(post_with_image())));
        api.expect_download_image()
            .returning(|_| Err(FetchError("Response is too large".to_string())));
        let event = message_event("ignored");

        let actions = preview_messages(&event, &link(), &api).await;

        let preview = Preview::new(&post(post_with_image()), &link());
        assert_eq!(
            actions,
            vec![Action::FirstSuccessful(vec![
                preview_with_images(&event, &preview, vec![Image::new(IMAGE_URL, "A cat")]),
                text_only_preview(&event, &preview),
            ])]
        );
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

        let actions = preview_messages(&message_event("ignored"), &link(), &api).await;

        match &actions[..] {
            [Action::FirstSuccessful(alternatives)] => match alternatives.last().unwrap() {
                Action::MessageChannel { message, .. } => {
                    assert_eq!(
                        message.get_text(),
                        "Mastodon post by @someone@other.example"
                    )
                }
                other => panic!("Expected a text only preview, got {other:?}"),
            },
            other => panic!("Expected alternative replies, got {other:?}"),
        }
    }
}
