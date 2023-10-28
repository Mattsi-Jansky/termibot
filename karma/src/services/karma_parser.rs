use regex::{CaptureMatches, Regex};
use lazy_static::lazy_static;

lazy_static! {
    static ref KARMA_MATCHER: Regex = Regex::new(r"([^\`\s]{2,})(--|\+\+)(^|\s|$)").unwrap();
}

pub struct KarmaCapture {
    pub name: String,
    pub is_increment: bool
}

pub fn get_captures(text: &str) -> Vec<KarmaCapture> {
    let mut result = vec![];

    for capture in KARMA_MATCHER.captures_iter(text) {
        let capture = capture.get(0).unwrap().as_str().trim();
        result.push(KarmaCapture {
            name: capture[..capture.len() - 2].to_string(),
            is_increment: capture[capture.len() - 2..].eq("++")
        })
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_no_captures_should_return_empty() {
        let result = get_captures("words words words");

        assert_eq!(0, result.len())
    }

    #[test]
    fn should_trim_spaces_and_pluses() {
        let result = get_captures(" sunnydays++ ");

        assert_eq!(1, result.len());
        assert_eq!(result.get(0).unwrap().name, "sunnydays");
    }
}