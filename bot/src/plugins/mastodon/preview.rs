use crate::plugins::mastodon::link::PostLink;
use crate::plugins::mastodon::mrkdwn;
use crate::plugins::mastodon::post::{MediaAttachment, MediaKind, Post};
use client::models::blocks::card::CardBlock;
use client::models::blocks::context::{ContextBlock, ContextElement};
use client::models::blocks::elements::button::{ButtonElement, ButtonStyle};
use client::models::blocks::elements::image::ImageElement;
use client::models::blocks::objects::text::Text;
use client::models::blocks::section::SectionBlock;
use client::models::blocks::text::HeaderBlock;
use client::models::blocks::Block;
use client::models::message_body::MessageBody;
use framework::actions::Image;

const SECTION_TEXT_LIMIT: usize = 3000;
const HEADER_TEXT_LIMIT: usize = 150;
const CARD_BODY_LIMIT: usize = 200;
const ALT_TEXT_LIMIT: usize = 2000;
const MAX_IMAGES: usize = 4;
const MAX_VIDEOS: usize = 4;
const SUPPORTED_IMAGE_EXTENSIONS: [&str; 4] = [".png", ".jpg", ".jpeg", ".gif"];

pub struct Preview {
    kind: Kind,
    handle: String,
    author: String,
    avatar_url: Option<String>,
    post_url: String,
    content_warning: Option<String>,
    text: String,
    images: Vec<Image>,
    videos: Vec<VideoCard>,
    sensitive: bool,
    unshowable_attachments: usize,
}

struct VideoCard {
    label: &'static str,
    watch_url: String,
    thumbnail_url: Option<String>,
    alt_text: String,
    description: Option<String>,
    duration: Option<String>,
}

#[derive(Clone, Copy)]
enum Kind {
    Post,
    QuotedPost,
}

enum Content {
    WithMedia,
    TextOnly,
}

impl Preview {
    pub fn new(post: &Post, link: &PostLink) -> Self {
        Self::build(post, link, Kind::Post)
    }

    pub fn for_quoted_post(post: &Post, link: &PostLink) -> Self {
        Self::build(post, link, Kind::QuotedPost)
    }

    fn build(post: &Post, link: &PostLink, kind: Kind) -> Self {
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
        let post_url = post.url.clone().unwrap_or_else(|| link.url.clone());
        let posted = mrkdwn::date(&post.created_at, &post_url);
        let author = match account.display_name.trim() {
            "" => format!("{profile_link} · {posted}"),
            name => format!("*{}* {profile_link} · {posted}", mrkdwn::escape(name)),
        };

        let mut images = vec![];
        let mut videos = vec![];
        let mut unshowable_attachments = 0;
        for attachment in &post.media_attachments {
            match attachment.kind {
                MediaKind::Image => match image_for(attachment) {
                    Some(image) if images.len() < MAX_IMAGES => images.push(image),
                    _ => unshowable_attachments += 1,
                },
                MediaKind::Video | MediaKind::Gifv if videos.len() < MAX_VIDEOS => {
                    videos.push(VideoCard::new(attachment, &post_url))
                }
                _ => unshowable_attachments += 1,
            }
        }

        Self {
            kind,
            handle,
            author,
            avatar_url: account
                .avatar_static
                .clone()
                .filter(|url| is_supported_image(url)),
            post_url,
            content_warning: Some(post.spoiler_text.trim())
                .filter(|warning| !warning.is_empty())
                .map(str::to_string),
            text: preview_text(post, kind),
            images,
            videos,
            sensitive: post.sensitive,
            unshowable_attachments,
        }
    }

    pub fn message(&self) -> MessageBody {
        self.build_message(Content::WithMedia)
    }

    pub fn text_only_message(&self) -> MessageBody {
        self.build_message(Content::TextOnly)
    }

    pub fn images(&self) -> Vec<Image> {
        if self.sensitive {
            vec![]
        } else {
            self.images.clone()
        }
    }

    fn build_message(&self, content: Content) -> MessageBody {
        let with_media = matches!(content, Content::WithMedia);
        let shows_media = with_media && !self.sensitive;

        let mut blocks = vec![self.author_block(with_media)];
        if let Some(warning) = &self.content_warning {
            blocks.push(HeaderBlock::new(&truncate_plain(
                &format!("CW: {warning}"),
                HEADER_TEXT_LIMIT,
            )));
        }
        if !self.text.is_empty() {
            blocks.push(SectionBlock::new_markdown(&self.text));
        }
        if shows_media {
            blocks.extend(self.videos.iter().map(VideoCard::block));
        }
        let hidden_media = if shows_media {
            0
        } else {
            self.images.len() + self.videos.len()
        };
        let hidden_attachments = self.unshowable_attachments + hidden_media;
        if hidden_attachments > 0 {
            blocks.push(self.hidden_attachments_note(hidden_attachments));
        }

        let summary = match self.kind {
            Kind::Post => "Mastodon post by",
            Kind::QuotedPost => "Quoted post by",
        };
        MessageBody::new(blocks, Some(format!("{summary} @{}", self.handle)))
            .expect("a preview always has blocks")
            .without_unfurling()
    }

    fn author_block(&self, with_avatar: bool) -> Block {
        let avatar = self
            .avatar_url
            .as_deref()
            .filter(|_| with_avatar)
            .map(|url| ContextElement::image(url, "Profile picture"));
        let label = match self.kind {
            Kind::Post => None,
            Kind::QuotedPost => Some(ContextElement::markdown("↳ Quoted post by")),
        };
        let author = ContextElement::markdown(&self.author);

        ContextBlock::new(label.into_iter().chain(avatar).chain([author]).collect()).into()
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

impl VideoCard {
    fn new(attachment: &MediaAttachment, post_url: &str) -> Self {
        let description = description_of(attachment);
        Self {
            label: if attachment.kind == MediaKind::Gifv {
                "GIF"
            } else {
                "Video"
            },
            watch_url: attachment
                .url
                .clone()
                .unwrap_or_else(|| post_url.to_string()),
            thumbnail_url: attachment
                .preview_url
                .clone()
                .filter(|url| is_supported_image(url)),
            alt_text: alt_text(description, "Video thumbnail"),
            description: description
                .map(|description| truncate_plain(description, CARD_BODY_LIMIT)),
            duration: attachment.duration_in_seconds().map(format_duration),
        }
    }

    fn block(&self) -> Block {
        let mut card = CardBlock::default()
            .with_title(Text::plain(self.label))
            .with_actions(vec![
                ButtonElement::link("▶ Watch", &self.watch_url).with_style(ButtonStyle::Primary)
            ]);
        if let Some(url) = &self.thumbnail_url {
            card = card.with_hero_image(ImageElement::from_url(url, &self.alt_text));
        }
        if let Some(duration) = &self.duration {
            card = card.with_subtitle(Text::plain(duration));
        }
        if let Some(description) = &self.description {
            card = card.with_body(Text::plain(description));
        }
        card.into()
    }
}

fn preview_text(post: &Post, kind: Kind) -> String {
    let shows_quoted_post = matches!(kind, Kind::Post) && post.quoted_post().is_some();
    let text = if shows_quoted_post {
        mrkdwn::from_html_without_quote_fallback(&post.content)
    } else {
        mrkdwn::from_html(&post.content)
    };
    let room_for_quote_marks = 2 * text.lines().count();
    let cut_short = mrkdwn::truncate(
        &text,
        SECTION_TEXT_LIMIT.saturating_sub(room_for_quote_marks),
    );
    mrkdwn::quote(&cut_short)
}

fn image_for(attachment: &MediaAttachment) -> Option<Image> {
    let url = [&attachment.url, &attachment.preview_url]
        .into_iter()
        .flatten()
        .find(|url| is_supported_image(url))?;
    Some(Image::new(
        url,
        &alt_text(description_of(attachment), "Image"),
    ))
}

fn description_of(attachment: &MediaAttachment) -> Option<&str> {
    attachment
        .description
        .as_deref()
        .map(str::trim)
        .filter(|description| !description.is_empty())
}

fn format_duration(seconds: f64) -> String {
    let total = seconds.round() as u64;
    let (hours, minutes, seconds) = (total / 3600, total % 3600 / 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
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
    use crate::plugins::mastodon::post::test_data::{
        post, post_json, post_quoting, quoted_post_json, real_post, real_video_post, QUOTE_FALLBACK,
    };
    use client::models::blocks::objects::text::TextBody;
    use serde_json::{json, Value};

    const AVATAR_URL: &str = "https://files.mastodon.social/avatars/sam.png";
    const POSTED: &str = "<!date^1793957400^{date_short_pretty} at {time}^https://mastodon.social/@sam/111|2026-11-06 09:30 UTC>";
    const QUOTED_POSTED: &str = "<!date^1793957400^{date_short_pretty} at {time}^https://other.example/@alex/222|2026-11-06 09:30 UTC>";
    const REAL_POSTED: &str = "<!date^1784197149^{date_short_pretty} at {time}^https://mastodon.social/@Mastodon/116929144390213579|2026-07-16 10:19 UTC>";
    const IMAGE_URL: &str = "https://files.mastodon.social/media/cat.png";
    const VIDEO_URL: &str = "https://files.mastodon.social/media/cat.mp4";
    const THUMBNAIL_URL: &str = "https://files.mastodon.social/media/cat-thumbnail.jpg";

    fn video(kind: &str) -> Value {
        json!({
            "type": kind,
            "url": VIDEO_URL,
            "preview_url": THUMBNAIL_URL,
            "description": "A cat jumping",
            "meta": {"original": {"duration": 80.662}}
        })
    }

    fn watch_button(url: &str) -> Vec<ButtonElement> {
        vec![ButtonElement::link("▶ Watch", url).with_style(ButtonStyle::Primary)]
    }

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
    fn post_quoting_a_post_that_is_shown_leaves_out_the_quote_fallback() {
        let preview = preview(post_quoting(
            json!({"state": "accepted", "quoted_status": quoted_post_json()}),
        ));

        assert_eq!(section_text(&preview.message()), "> Look at this");
    }

    #[test]
    fn post_quoting_a_post_that_cannot_be_shown_keeps_the_quote_fallback() {
        let preview = preview(post_quoting(
            json!({"state": "deleted", "quoted_status": null}),
        ));

        assert_eq!(
            section_text(&preview.message()),
            "> RE: <https://other.example/@alex/222|other.example/@alex/222>\n>\n> Look at this"
        );
    }

    #[test]
    fn quoted_post_keeps_the_fallback_for_its_own_quote_which_is_not_shown() {
        let mut quoted =
            post_quoting(json!({"state": "accepted", "quoted_status": quoted_post_json()}));
        quoted["content"] = json!(format!("{QUOTE_FALLBACK}<p>The original</p>"));
        let preview = Preview::for_quoted_post(
            &post(quoted),
            &PostLink::find_in("https://mastodon.social/@sam/111").unwrap(),
        );

        assert!(section_text(&preview.message()).starts_with("> RE: "));
    }

    #[test]
    fn quoted_post_preview_is_labelled_as_the_quoted_post() {
        let quoted = post(quoted_post_json());
        let preview = Preview::for_quoted_post(
            &quoted,
            &PostLink::find_in("https://mastodon.social/@sam/111").unwrap(),
        );
        let label = ContextElement::markdown("↳ Quoted post by");
        let author = ContextElement::markdown(&format!(
            "*Alex* <https://other.example/@alex|@alex@other.example> · {QUOTED_POSTED}"
        ));

        assert_eq!(
            preview.message().get_blocks()[0],
            ContextBlock::new(vec![
                label,
                ContextElement::image(AVATAR_URL, "Profile picture"),
                author,
            ])
            .into()
        );
        assert_eq!(
            preview.text_only_message().get_blocks()[0],
            ContextBlock::new(vec![
                ContextElement::markdown("↳ Quoted post by"),
                ContextElement::markdown(&format!(
                    "*Alex* <https://other.example/@alex|@alex@other.example> · {QUOTED_POSTED}"
                )),
            ])
            .into()
        );
        assert_eq!(
            preview.message().get_text(),
            "Quoted post by @alex@other.example"
        );
    }

    #[test]
    fn message_shows_author_with_avatar_then_text() {
        let message = preview(post_json()).message();

        assert_eq!(
            message.get_blocks(),
            &vec![
                ContextBlock::new(vec![
                    ContextElement::image(AVATAR_URL, "Profile picture"),
                    ContextElement::markdown(&format!(
                        "*Sam* <https://mastodon.social/@sam|@sam@mastodon.social> · {POSTED}"
                    )),
                ])
                .into(),
                SectionBlock::new_markdown("> Hello"),
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
            format!("*Sam* <https://mastodon.social/@sam|@sam@other.example> · {POSTED}")
        );
    }

    #[test]
    fn author_without_a_display_name_is_shown_by_handle() {
        let mut json = post_json();
        json["account"]["display_name"] = json!("");

        assert_eq!(
            author_markdown(&preview(json).message()),
            format!("<https://mastodon.social/@sam|@sam@mastodon.social> · {POSTED}")
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
                SectionBlock::new_markdown("> Hello"),
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
    fn long_text_with_many_paragraphs_is_quoted_and_still_fits_in_slack() {
        let mut json = post_json();
        json["content"] = json!(format!("<p>{}</p>", "a".repeat(40)).repeat(100));

        let text = section_text(&preview(json).message()).to_string();

        assert!(text.chars().count() <= SECTION_TEXT_LIMIT);
        assert!(text.lines().all(|line| line.starts_with('>')), "{text}");
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
    fn videos_are_shown_as_cards_with_a_watch_button() {
        let preview = preview(with_attachments(json!([video("video")])));

        assert!(preview.images().is_empty());
        assert_eq!(
            preview.message().get_blocks().last().unwrap(),
            &CardBlock::default()
                .with_hero_image(ImageElement::from_url(THUMBNAIL_URL, "A cat jumping"))
                .with_title(Text::plain("Video"))
                .with_subtitle(Text::plain("1:21"))
                .with_body(Text::plain("A cat jumping"))
                .with_actions(watch_button(VIDEO_URL))
                .into()
        );
    }

    #[test]
    fn gifs_are_labelled_as_gifs() {
        let message = preview(with_attachments(json!([video("gifv")]))).message();

        assert!(matches!(
            message.get_blocks().last().unwrap(),
            Block::Card(card) if card.title == Some(Text::plain("GIF"))
        ));
    }

    #[test]
    fn video_cards_leave_out_what_the_post_does_not_have() {
        let preview = preview(with_attachments(json!([{
            "type": "video",
            "url": VIDEO_URL,
            "preview_url": "https://files.mastodon.social/media/cat-thumbnail.webp"
        }])));

        assert_eq!(
            preview.message().get_blocks().last().unwrap(),
            &CardBlock::default()
                .with_title(Text::plain("Video"))
                .with_actions(watch_button(VIDEO_URL))
                .into()
        );
    }

    #[test]
    fn videos_without_their_own_url_are_watched_on_the_post() {
        let message = preview(with_attachments(json!([{"type": "video", "url": null}]))).message();

        assert!(matches!(
            message.get_blocks().last().unwrap(),
            Block::Card(card) if card.actions == Some(watch_button("https://mastodon.social/@sam/111"))
        ));
    }

    #[test]
    fn long_video_descriptions_are_cut_short_to_fit_in_the_card() {
        let mut video = video("video");
        video["description"] = json!("a".repeat(300));

        let message = preview(with_attachments(json!([video]))).message();

        match message.get_blocks().last().unwrap() {
            Block::Card(card) => match &card.body {
                Some(Text::PlainText(TextBody { text })) => {
                    assert_eq!(text.chars().count(), CARD_BODY_LIMIT)
                }
                other => panic!("Expected the video's description, got {other:?}"),
            },
            other => panic!("Expected a video card, got {other:?}"),
        }
    }

    #[test]
    fn sensitive_videos_are_noted_instead_of_shown() {
        let mut json = with_attachments(json!([video("video")]));
        json["sensitive"] = json!(true);

        let message = preview(json).message();

        assert!(!message
            .get_blocks()
            .iter()
            .any(|block| matches!(block, Block::Card(_))));
        assert_eq!(
            message.get_blocks().last().unwrap(),
            &note("1 attachment marked sensitive, <https://mastodon.social/@sam/111|view on Mastodon>")
        );
    }

    #[test]
    fn text_only_message_notes_videos_instead_of_showing_them() {
        let message = preview(with_attachments(json!([video("video")]))).text_only_message();

        assert!(!message
            .get_blocks()
            .iter()
            .any(|block| matches!(block, Block::Card(_))));
        assert_eq!(
            message.get_blocks().last().unwrap(),
            &note("1 attachment not shown, <https://mastodon.social/@sam/111|view on Mastodon>")
        );
    }

    #[test]
    fn video_lengths_are_shown_in_minutes_and_seconds() {
        assert_eq!(format_duration(5.2), "0:05");
        assert_eq!(format_duration(80.662), "1:21");
        assert_eq!(format_duration(3725.0), "1:02:05");
    }

    #[test]
    fn real_video_post_is_previewed_as_a_card() {
        let preview = Preview::new(
            &real_video_post(),
            &PostLink::find_in(
                "https://front-end.social/@jaffathecake@mastodon.social/116645868006890634",
            )
            .unwrap(),
        );

        assert_eq!(
            preview.message().get_blocks().last().unwrap(),
            &CardBlock::default()
                .with_hero_image(ImageElement::from_url(
                    "https://cdn.masto.host/frontendsocial/cache/media_attachments/files/116/645/867/820/386/059/small/bc40b1a74d593ed8.png",
                    "Video thumbnail"
                ))
                .with_title(Text::plain("Video"))
                .with_subtitle(Text::plain("1:21"))
                .with_actions(watch_button(
                    "https://cdn.masto.host/frontendsocial/cache/media_attachments/files/116/645/867/820/386/059/original/bc40b1a74d593ed8.mp4"
                ))
                .into()
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
                ContextBlock::new(vec![ContextElement::markdown(&format!(
                    "*Sam* <https://mastodon.social/@sam|@sam@mastodon.social> · {POSTED}"
                ))])
                .into(),
                SectionBlock::new_markdown("> Hello"),
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
            format!("*Mastodon* <https://mastodon.social/@Mastodon|@Mastodon@mastodon.social> · {REAL_POSTED}")
        );
        assert!(text.starts_with(
            "> New in Mastodon 4.6: Newsletters 📬\n>\n> Mastodon servers now support email newsletters"
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
