use html_escape::decode_html_entities;
use tl::{HTMLTag, Node, NodeHandle, Parser, ParserOptions};

pub fn from_html(html: &str) -> String {
    let Ok(dom) = tl::parse(html, ParserOptions::default()) else {
        return escape(html);
    };
    let mut writer = MrkdwnWriter::new(dom.parser());
    writer.write_nodes(dom.children());
    writer.output.trim().to_string()
}

pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Cuts the text short without leaving half of a link or an escaped character behind
pub fn truncate(mrkdwn: &str, max_chars: usize) -> String {
    if mrkdwn.chars().count() <= max_chars {
        return mrkdwn.to_string();
    }

    let mut cut: String = mrkdwn.chars().take(max_chars.saturating_sub(1)).collect();
    for (start, end) in [('<', '>'), ('&', ';')] {
        if let Some(position) = cut.rfind(start) {
            if !cut[position..].contains(end) {
                cut.truncate(position);
            }
        }
    }
    cut.truncate(cut.trim_end().len());
    cut.push('…');
    cut
}

struct MrkdwnWriter<'p, 'buf> {
    parser: &'p Parser<'buf>,
    output: String,
}

impl<'p, 'buf> MrkdwnWriter<'p, 'buf> {
    fn new(parser: &'p Parser<'buf>) -> Self {
        Self {
            parser,
            output: String::new(),
        }
    }

    fn write_nodes(&mut self, handles: &[NodeHandle]) {
        for handle in handles {
            match handle.get(self.parser) {
                Some(Node::Tag(tag)) => self.write_tag(tag),
                Some(Node::Raw(text)) => self
                    .output
                    .push_str(&escape(&decode_html_entities(&text.as_utf8_str()))),
                _ => {}
            }
        }
    }

    fn write_children(&mut self, tag: &HTMLTag) {
        self.write_nodes(tag.children().top().as_slice());
    }

    fn write_tag(&mut self, tag: &HTMLTag) {
        match tag.name().as_utf8_str().to_ascii_lowercase().as_str() {
            "br" => self.output.push('\n'),
            "p" => {
                self.start_paragraph();
                self.write_children(tag);
            }
            "li" => {
                self.start_line();
                self.output.push_str("• ");
                self.write_children(tag);
            }
            "a" => self.write_link(tag),
            "span" if has_class(tag, "invisible") => {}
            "span" if has_class(tag, "ellipsis") => {
                self.write_children(tag);
                self.output.push('…');
            }
            "b" | "strong" => self.write_wrapped(tag, "*"),
            "i" | "em" => self.write_wrapped(tag, "_"),
            "del" | "s" => self.write_wrapped(tag, "~"),
            "code" => self.write_wrapped(tag, "`"),
            _ => self.write_children(tag),
        }
    }

    fn start_paragraph(&mut self) {
        if !self.output.is_empty() {
            self.output.push_str("\n\n");
        }
    }

    fn start_line(&mut self) {
        if !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn write_link(&mut self, tag: &HTMLTag) {
        let mut label = MrkdwnWriter::new(self.parser);
        label.write_children(tag);

        match attribute(tag, "href") {
            Some(href) if !label.output.is_empty() => {
                self.output
                    .push_str(&format!("<{}|{}>", escape(&href), label.output))
            }
            Some(href) => self.output.push_str(&format!("<{}>", escape(&href))),
            None => self.output.push_str(&label.output),
        }
    }

    fn write_wrapped(&mut self, tag: &HTMLTag, marker: &str) {
        self.output.push_str(marker);
        self.write_children(tag);
        self.output.push_str(marker);
    }
}

fn attribute(tag: &HTMLTag, name: &str) -> Option<String> {
    tag.attributes()
        .get(name)
        .flatten()
        .map(|value| decode_html_entities(&value.as_utf8_str()).into_owned())
}

fn has_class(tag: &HTMLTag, class: &str) -> bool {
    tag.attributes()
        .class_iter()
        .is_some_and(|mut classes| classes.any(|name| name == class))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_separate_paragraphs_and_keep_line_breaks() {
        assert_eq!(
            from_html("<p>First line<br>second line</p><p>Next paragraph</p>"),
            "First line\nsecond line\n\nNext paragraph"
        );
    }

    #[test]
    fn should_decode_html_entities_and_escape_slack_markup() {
        assert_eq!(
            from_html("<p>We&#39;re &lt;3 &amp; you &gt; me</p>"),
            "We're &lt;3 &amp; you &gt; me"
        );
    }

    #[test]
    fn should_show_shortened_links_as_mastodon_does() {
        assert_eq!(
            from_html(
                r#"<p>See <a href="https://blog.example.org/2026/06/a-long-post/" rel="nofollow"><span class="invisible">https://</span><span class="ellipsis">blog.example.org/2026/06/</span><span class="invisible">a-long-post/</span></a></p>"#
            ),
            "See <https://blog.example.org/2026/06/a-long-post/|blog.example.org/2026/06/…>"
        );
    }

    #[test]
    fn should_link_mentions_and_hashtags() {
        assert_eq!(
            from_html(
                r#"<p><span class="h-card"><a href="https://mas.to/@sam" class="u-url mention">@<span>sam</span></a></span> loves <a href="https://mas.to/tags/rust" class="mention hashtag" rel="tag">#<span>rust</span></a></p>"#
            ),
            "<https://mas.to/@sam|@sam> loves <https://mas.to/tags/rust|#rust>"
        );
    }

    #[test]
    fn should_escape_link_urls() {
        assert_eq!(
            from_html(r#"<a href="https://example.com/?a=1&amp;b=2">link</a>"#),
            "<https://example.com/?a=1&amp;b=2|link>"
        );
    }

    #[test]
    fn should_format_lists_and_emphasis() {
        assert_eq!(
            from_html(
                "<p>Shopping:</p><ul><li><b>bread</b></li><li><i>milk</i></li></ul><p>Done</p>"
            ),
            "Shopping:\n• *bread*\n• _milk_\n\nDone"
        );
    }

    #[test]
    fn should_leave_short_text_alone() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn should_cut_long_text_short() {
        assert_eq!(truncate("hello world", 8), "hello w…");
    }

    #[test]
    fn should_not_leave_half_a_link_when_cutting() {
        assert_eq!(truncate("see <https://example.com|example>", 15), "see…");
    }

    #[test]
    fn should_not_leave_half_an_escaped_character_when_cutting() {
        assert_eq!(truncate("fish &amp; chips", 8), "fish…");
    }
}
