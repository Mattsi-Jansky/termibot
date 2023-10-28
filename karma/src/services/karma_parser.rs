use regex::{CaptureMatches, Regex};
use lazy_static::lazy_static;

lazy_static! {
    static ref KARMA_MATCHER: Regex = Regex::new(r"([^\`\s]{2,})(--|\+\+)(^|\s|$)").unwrap();
}

pub fn get_captures(text: &String) -> CaptureMatches {
    KARMA_MATCHER.captures_iter(&text[..])
}