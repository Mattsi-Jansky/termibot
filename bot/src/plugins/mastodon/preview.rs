use crate::plugins::mastodon::link::PostLink;
use crate::plugins::mastodon::mrkdwn;
use crate::plugins::mastodon::post::{MediaAttachment, MediaKind, Post};
use client::models::blocks::context::{ContextBlock, ContextElement};
use client::models::blocks::section::SectionBlock;
use client::models::blocks::text::HeaderBlock;
use client::models::blocks::Block;
use client::models::message_body::MessageBody;
use framework::actions::Image;

const SECTION_TEXT_LIMIT: usize = 3000;
const HEADER_TEXT_LIMIT: usize = 150;
const ALT_TEXT_LIMIT: usize = 2000;
const MAX_IMAGES: usize = 4;
const SUPPORTED_IMAGE_EXTENSIONS: [&str; 4] = [".png", ".jpg", ".jpeg", ".gif"];

pub struct Preview {
    handle: String,
    author: String,
    avatar_url: Option<String>,
    post_url: String,
    content_warning: Option<String>,
    text: String,
    images: Vec<Image>,
    sensitive: bool,
    unshowable_attachments: usize,
}

impl Preview {
    pub fn new(post: &Post, link: &PostLink) -> Self {
        let account = &post.account;
        let handle = if account.acct.contains('@') {
            account.acct.clone()
        } else {
            format!("{}@{}", account.acct, link.host)
        };
        let profile_link = format!(
            "<{}|@{}>",
            mrkdwn::escape(&account.url),
            mrkdwn::escape(&handle)
        );
        let author = match account.display_name.trim() {
            "" => profile_link,
            name => format!("*{}* {profile_link}", mrkdwn::escape(name)),
        };

        let mut images = vec![];
        let mut unshowable_attachments = 0;
        for attachment in &post.media_attachments {
            match image_for(attachment) {
                Some(image) if images.len() < MAX_IMAGES => images.push(image),
                _ => unshowable_attachments += 1,
            }
        }

        Self {
            handle,
            author,
            avatar_url: account
                .avatar_static
                .clone()
                .filter(|url| is_supported_image(url)),
            post_url: post.url.clone().unwrap_or_else(|| link.url.clone()),
            content_warning: Some(post.spoiler_text.trim())
                .filter(|warning| !warning.is_empty())
                .map(str::to_string),
            text: mrkdwn::truncate(&mrkdwn::from_html(&post.content), SECTION_TEXT_LIMIT),
            images,
            sensitive: post.sensitive,
            unshowable_attachments,
        }
    }

    pub fn message(&self) -> MessageBody {
        let hidden_images = if self.sensitive { self.images.len() } else { 0 };
        self.build_message(true, self.unshowable_attachments + hidden_images)
    }

    pub fn text_only_message(&self) -> MessageBody {
        self.build_message(false, self.unshowable_attachments + self.images.len())
    }

    pub fn images(&self) -> Vec<Image> {
        if self.sensitive {
            vec![]
        } else {
            self.images.clone()
        }
    }

    fn build_message(&self, with_avatar: bool, hidden_attachments: usize) -> MessageBody {
        let mut blocks = vec![self.author_block(with_avatar)];
        if let Some(warning) = &self.content_warning {
            blocks.push(HeaderBlock::new(&truncate_plain(
                &format!("CW: {warning}"),
                HEADER_TEXT_LIMIT,
            )));
        }
        if !self.text.is_empty() {
            blocks.push(SectionBlock::new_markdown(&self.text));
        }
        if hidden_attachments > 0 {
            blocks.push(self.hidden_attachments_note(hidden_attachments));
        }

        MessageBody::new(blocks, Some(format!("Mastodon post by @{}", self.handle)))
            .expect("a preview always has blocks")
            .without_unfurling()
    }

    fn author_block(&self, with_avatar: bool) -> Block {
        let avatar = self
            .avatar_url
            .as_deref()
            .filter(|_| with_avatar)
            .map(|url| ContextElement::image(url, "Profile picture"));
        let author = ContextElement::markdown(&self.author);

        ContextBlock::new(avatar.into_iter().chain([author]).collect()).into()
    }

    fn hidden_attachments_note(&self, count: usize) -> Block {
        let attachments = if count == 1 {
            "attachment"
        } else {
            "attachments"
        };
        let reason = if self.sensitive {
            "marked sensitive"
        } else {
            "not shown"
        };

        ContextBlock::new(vec![ContextElement::markdown(&format!(
            "{count} {attachments} {reason}, <{}|view on Mastodon>",
            mrkdwn::escape(&self.post_url)
        ))])
        .into()
    }
}

fn image_for(attachment: &MediaAttachment) -> Option<Image> {
    let description = attachment
        .description
        .as_deref()
        .map(str::trim)
        .filter(|description| !description.is_empty());

    match attachment.kind {
        MediaKind::Image => {
            let url = [&attachment.url, &attachment.preview_url]
                .into_iter()
                .flatten()
                .find(|url| is_supported_image(url))?;
            Some(Image::new(url, &alt_text(description, "Image")))
        }
        MediaKind::Video | MediaKind::Gifv => {
            let url = attachment
                .preview_url
                .as_deref()
                .filter(|url| is_supported_image(url))?;
            Some(Image::new(url, &alt_text(description, "Video thumbnail")).with_title("Video"))
        }
        MediaKind::Audio | MediaKind::Unknown => None,
    }
}

fn alt_text(description: Option<&str>, fallback: &str) -> String {
    truncate_plain(description.unwrap_or(fallback), ALT_TEXT_LIMIT)
}

fn is_supported_image(url: &str) -> bool {
    let path = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .to_ascii_lowercase();
    SUPPORTED_IMAGE_EXTENSIONS
        .iter()
        .any(|extension| path.ends_with(extension))
}

fn truncate_plain(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::mastodon::post::test_data::{post, post_json, real_post};
    use client::models::blocks::objects::text::{Text, TextBody};
    use serde_json::{json, Value};

    const AVATAR_URL: &str = "https://files.mastodon.social/avatars/sam.png";
    const IMAGE_URL: &str = "https://files.mastodon.social/media/cat.png";

    fn preview(json: Value) -> Preview {
        Preview::new(
            &post(json),
            &PostLink::find_in("https://mastodon.social/@sam/111").unwrap(),
        )
    }

    fn with_attachments(attachments: Value) -> Value {
        let mut json = post_json();
        json["media_attachments"] = attachments;
        json
    }

    fn author_markdown(message: &MessageBody) -> &str {
        match &message.get_blocks()[0] {
            Block::Context(ContextBlock { elements }) => match elements.last() {
                Some(ContextElement::Text(Text::Markdown(TextBody { text }))) => text,
                other => panic!("Expected the author's name, got {other:?}"),
            },
            other => panic!("Expected the author line, got {other:?}"),
        }
    }

    fn section_text(message: &MessageBody) -> &str {
        message
            .get_blocks()
            .iter()
            .find_map(|block| match block {
                Block::Section(SectionBlock {
                    text: Some(Text::Markdown(TextBody { text })),
                    ..
                }) => Some(text.as_str()),
                _ => None,
            })
            .expect("message should have a text section")
    }

    fn note(text: &str) -> Block {
        ContextBlock::new(vec![ContextElement::markdown(text)]).into()
    }

    #[test]
    fn message_shows_author_with_avatar_then_text() {
        let message = preview(post_json()).message();

        assert_eq!(
            message.get_blocks(),
            &vec![
                ContextBlock::new(vec![
                    ContextElement::image(AVATAR_URL, "Profile picture"),
                    ContextElement::markdown(
                        "*Sam* <https://mastodon.social/@sam|@sam@mastodon.social>"
                    ),
                ])
                .into(),
                SectionBlock::new_markdown("Hello"),
            ]
        );
    }

    #[test]
    fn messages_turn_off_unfurling_and_mention_the_author_in_their_notification_text() {
        let preview = preview(post_json());

        for message in [preview.message(), preview.text_only_message()] {
            assert!(!message.unfurls());
            assert_eq!(message.get_text(), "Mastodon post by @sam@mastodon.social");
        }
    }

    #[test]
    fn author_from_another_server_keeps_their_own_server_in_their_handle() {
        let mut json = post_json();
        json["account"]["acct"] = json!("sam@other.example");

        assert_eq!(
            author_markdown(&preview(json).message()),
            "*Sam* <https://mastodon.social/@sam|@sam@other.example>"
        );
    }

    #[test]
    fn author_without_a_display_name_is_shown_by_handle() {
        let mut json = post_json();
        json["account"]["display_name"] = json!("");

        assert_eq!(
            author_markdown(&preview(json).message()),
            "<https://mastodon.social/@sam|@sam@mastodon.social>"
        );
    }

    #[test]
    fn author_avatar_is_left_out_when_slack_cannot_show_it() {
        let mut json = post_json();
        json["account"]["avatar_static"] = json!("https://files.mastodon.social/avatars/sam.webp");

        let message = preview(json).message();

        assert!(matches!(
            &message.get_blocks()[0],
            Block::Context(ContextBlock { elements }) if elements.len() == 1
        ));
    }

    #[test]
    fn content_warning_is_shown_as_a_heading_before_the_text() {
        let mut json = post_json();
        json["spoiler_text"] = json!("Spoilers for the finale");

        let message = preview(json).message();

        assert_eq!(
            &message.get_blocks()[1..],
            &[
                HeaderBlock::new("CW: Spoilers for the finale"),
                SectionBlock::new_markdown("Hello"),
            ]
        );
    }

    #[test]
    fn long_text_is_cut_short_to_fit_in_slack() {
        let mut json = post_json();
        json["content"] = json!(format!("<p>{}</p>", "a".repeat(4000)));

        let text = section_text(&preview(json).message()).to_string();

        assert_eq!(text.chars().count(), SECTION_TEXT_LIMIT);
        assert!(text.ends_with('…'));
    }

    #[test]
    fn images_are_shown_with_their_descriptions() {
        let preview = preview(with_attachments(json!([
            {"type": "image", "url": IMAGE_URL, "description": "A cat"},
            {"type": "image", "url": "https://files.mastodon.social/media/dog.jpg"}
        ])));

        assert_eq!(
            preview.images(),
            vec![
                Image::new(IMAGE_URL, "A cat"),
                Image::new("https://files.mastodon.social/media/dog.jpg", "Image"),
            ]
        );
    }

    #[test]
    fn images_in_formats_slack_cannot_show_use_their_preview_instead() {
        let preview = preview(with_attachments(json!([{
            "type": "image",
            "url": "https://files.mastodon.social/media/original/cat.webp",
            "preview_url": "https://files.mastodon.social/media/small/cat.png"
        }])));

        assert_eq!(
            preview.images()[0].url,
            "https://files.mastodon.social/media/small/cat.png"
        );
    }

    #[test]
    fn videos_are_shown_as_thumbnails() {
        let preview = preview(with_attachments(json!([{
            "type": "video",
            "url": "https://files.mastodon.social/media/cat.mp4",
            "preview_url": "https://files.mastodon.social/media/cat-thumbnail.jpg",
            "description": "A cat jumping"
        }])));

        assert_eq!(
            preview.images(),
            vec![Image::new(
                "https://files.mastodon.social/media/cat-thumbnail.jpg",
                "A cat jumping"
            )
            .with_title("Video")]
        );
    }

    #[test]
    fn attachments_that_cannot_be_shown_are_noted_after_the_text() {
        let message = preview(with_attachments(json!([
            {"type": "audio", "url": "https://files.mastodon.social/media/song.mp3"}
        ])))
        .message();

        assert_eq!(
            message.get_blocks().last().unwrap(),
            &note("1 attachment not shown, <https://mastodon.social/@sam/111|view on Mastodon>")
        );
    }

    #[test]
    fn only_the_first_four_images_are_shown() {
        let image = json!({"type": "image", "url": IMAGE_URL});
        let preview = preview(with_attachments(json!([image, image, image, image, image])));

        assert_eq!(preview.images().len(), 4);
        assert_eq!(
            preview.message().get_blocks().last().unwrap(),
            &note("1 attachment not shown, <https://mastodon.social/@sam/111|view on Mastodon>")
        );
    }

    #[test]
    fn sensitive_images_are_not_shown_and_are_noted_instead() {
        let mut json = with_attachments(json!([
            {"type": "image", "url": IMAGE_URL},
            {"type": "image", "url": IMAGE_URL}
        ]));
        json["sensitive"] = json!(true);
        let preview = preview(json);

        assert!(preview.images().is_empty());
        assert_eq!(
            preview.message().get_blocks().last().unwrap(),
            &note("2 attachments marked sensitive, <https://mastodon.social/@sam/111|view on Mastodon>")
        );
    }

    #[test]
    fn text_only_message_leaves_out_the_avatar_and_notes_the_images() {
        let message = preview(with_attachments(json!([
            {"type": "image", "url": IMAGE_URL}
        ])))
        .text_only_message();

        assert_eq!(
            message.get_blocks(),
            &vec![
                ContextBlock::new(vec![ContextElement::markdown(
                    "*Sam* <https://mastodon.social/@sam|@sam@mastodon.social>"
                )])
                .into(),
                SectionBlock::new_markdown("Hello"),
                note("1 attachment not shown, <https://mastodon.social/@sam/111|view on Mastodon>"),
            ]
        );
    }

    #[test]
    fn real_post_is_previewed_with_its_text_and_image() {
        let preview = Preview::new(
            &real_post(),
            &PostLink::find_in("https://mastodon.social/@Mastodon/116929144390213579").unwrap(),
        );
        let message = preview.message();
        let text = section_text(&message);

        assert_eq!(
            author_markdown(&message),
            "*Mastodon* <https://mastodon.social/@Mastodon|@Mastodon@mastodon.social>"
        );
        assert!(text.starts_with(
            "New in Mastodon 4.6: Newsletters 📬\n\nMastodon servers now support email newsletters"
        ));
        assert!(text.contains("We're building Mastodon"));
        assert!(text.ends_with(
            "<https://blog.joinmastodon.org/2026/06/mastodon-4.6/|blog.joinmastodon.org/2026/06/…>"
        ));
        assert_eq!(preview.images().len(), 1);
        assert!(preview.images()[0]
            .alt_text
            .starts_with("The Mastodon profile page for Sage"));
    }
}
