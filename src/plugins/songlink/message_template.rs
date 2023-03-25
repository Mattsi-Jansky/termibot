use slack_morphism::{SlackMessageContent, SlackMessageTemplate};

#[derive(Debug, Clone)]
pub struct SongLinkMessageTemplate {
    pub url: String,
}

impl SlackMessageTemplate for SongLinkMessageTemplate {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new().with_text(format!("{}", self.url))
    }
}
