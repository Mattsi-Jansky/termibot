use regex::{CaptureMatches, Regex};
use lazy_static::lazy_static;

lazy_static! {
    static ref KARMA_MATCHER: Regex = Regex::new(r"([^\`\s]{2,})(--|\+\+)(^|\s|$)").unwrap();
}

#[derive(Debug,Eq,PartialEq)]
pub struct KarmaCapture {
    pub name: String,
    pub is_increment: bool,
    pub reason: Option<String>
}

impl KarmaCapture {
    pub fn new(name: String, is_increment: bool, reason: Option<String>) -> Self {
        Self { name, is_increment, reason }
    }
}

pub fn get_captures(text: &str) -> Vec<KarmaCapture> {
    let mut result = vec![];

    for capture in KARMA_MATCHER.captures_iter(text) {
        let capture = capture.get(0).unwrap().as_str().trim();
        result.push(KarmaCapture {
            name: capture[..capture.len() - 2].to_string(),
            is_increment: capture[capture.len() - 2..].eq("++"),
            reason: None
        })
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse_tests {
        ($(($name:ident, $input:expr, $expected:expr)),*) => {
            $(#[test]
            fn $name() {
                let expected = $expected;
                let result = get_captures($input);

                assert_eq!(expected.len(), result.len());
                for (expectation,actual) in expected.iter().zip(result.iter()) {
                    assert_eq!(expectation, actual)
                }
            })*
        }
    }

    parse_tests!{
        (given_no_captures_should_return_empty, "words words words", Vec::<KarmaCapture>::new()),
        (should_trim_spaces_and_pluses, " sunnydays++ ", vec![ KarmaCapture::new("sunnydays".to_string(), true, None)]),
        (should_trim_spaces_and_minuses, " rainydays-- ", vec![ KarmaCapture::new("rainydays".to_string(), false, None)]),
        (should_maintain_capitalisation, " RainyDays-- ", vec![ KarmaCapture::new("RainyDays".to_string(), false, None)]),
        (should_parse_emoji, ":smile:++", vec![ KarmaCapture::new(":smile:".to_string(), true, None)])
    }
}
