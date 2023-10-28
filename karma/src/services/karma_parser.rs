use regex::{CaptureMatches, Regex};
use lazy_static::lazy_static;

lazy_static! {
    static ref KARMA_MATCHER: Regex = Regex::new(r"([^\`\s]{2,})(--|\+\+)(^|\s|$)").unwrap();
}

pub struct KarmaCapture {
    pub name: String,
    pub is_increment: bool
}

pub fn get_captures(text: &String) -> Vec<KarmaCapture> {
    let mut result = vec![];

    for capture in KARMA_MATCHER.captures_iter(&text[..]) {
        let capture = capture.get(0).unwrap().as_str();
        result.push(KarmaCapture {
            name: capture[..capture.len() - 2].to_string(),
            is_increment: capture[capture.len() - 2..].eq("++")
        })
    }

    result
}
