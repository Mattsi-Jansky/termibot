use regex::{CaptureMatches, Captures, Match, Regex};
use lazy_static::lazy_static;
use tracing::error;

lazy_static! {
    static ref KARMA_MATCHER: Regex = Regex::new(r"([^`\-\+\s]{2,})(--|\+\+)(\s|$|\n|\+|\-)").unwrap();
    static ref PREFORMATTED_BLOCK_MATCHER: Regex = Regex::new(r"\`[^\`]*\`").unwrap();
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
    let preformatted_blocks = PREFORMATTED_BLOCK_MATCHER.captures_iter(text)
        .map(|block| block.get(0).unwrap())
        .collect::<Vec<Match>>();

    for capture in KARMA_MATCHER.captures_iter(text) {
        if !is_in_preformatted_block(&preformatted_blocks, &capture) {
            let name = capture.get(1).unwrap().as_str().trim();
            result.push(KarmaCapture {
                name: name.to_string(),
                is_increment: capture.get(2).unwrap().as_str().trim().eq("++"),
                reason: None
            })
        }
    }

    result
}

fn is_in_preformatted_block(preformatted_blocks: &Vec<Match>, capture: &Captures) -> bool {
    let mut result = false;
    let capture = capture.get(0).unwrap();

    for block in preformatted_blocks {
        println!("block {:?}\ncaptr {:?}", block, capture);
        if capture.start() > block.start() && capture.end() < block.end() {
            result = true;
        }
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

    parse_tests! {
        (given_no_captures_should_return_empty, "words words words", Vec::<KarmaCapture>::new()),
        (should_trim_spaces_and_pluses, " sunnydays++ ", vec![ KarmaCapture::new("sunnydays".to_string(), true, None)]),
        (should_trim_spaces_and_minuses, " rainydays-- ", vec![ KarmaCapture::new("rainydays".to_string(), false, None)]),
        (should_maintain_capitalisation, " RainyDays-- ", vec![ KarmaCapture::new("RainyDays".to_string(), false, None)]),
        (should_parse_emoji, ":smile:++", vec![ KarmaCapture::new(":smile:".to_string(), true, None)]),
        (should_isolate_name_after_errata, "I like to play chess++", vec![ KarmaCapture::new("chess".to_string(), true, None)]),
        (should_isolate_name_before_errata, "chess-- is really difficult", vec![ KarmaCapture::new("chess".to_string(), false, None)]),
        (should_isolate_name_surrounded_by__errata, "I like rainydays++ they are very cosy", vec![ KarmaCapture::new("rainydays".to_string(), true, None)]),
        (should_isolate_emoji_after_errata, ":smile:++ and errata", vec![ KarmaCapture::new(":smile:".to_string(), true, None)]),
        (should_isolate_emoji_before_errata, "errata and :smile:--", vec![ KarmaCapture::new(":smile:".to_string(), false, None)]),
        (should_isolate_name_with_too_many_pluses, "sunnydays+++", vec![ KarmaCapture::new("sunnydays".to_string(), true, None)]),
        (should_isolate_name_with_too_many_minuses, "rainydays---", vec![ KarmaCapture::new("rainydays".to_string(), false, None)]),
        (should_isolate_multiline_before, "rainydays\nsunnydays++", vec![ KarmaCapture::new("sunnydays".to_string(), true, None)]),
        (should_isolate_multiline_after, "rainydays--\nsunnydays", vec![ KarmaCapture::new("rainydays".to_string(), false, None)]),
        (should_isolate_at_start_of_string, "this++ is a matching phrase", vec![ KarmaCapture::new("this".to_string(), true, None)]),
        (should_isolate_at_end_of_string, "this is a matching phrase++", vec![ KarmaCapture::new("phrase".to_string(), true, None)]),
        (given_no_chars_before_pluses_should_return_empty, "++", Vec::<KarmaCapture>::new()),
        (given_no_chars_before_minuses_should_return_empty, "--", Vec::<KarmaCapture>::new()),
        (given_newline_before_pluses_should_return_empty, "hello\n++", Vec::<KarmaCapture>::new()),
        (given_four_pluses_should_return_empty, "++++", Vec::<KarmaCapture>::new()),
        (given_five_minuses_should_return_empty, "-----", Vec::<KarmaCapture>::new()),
        (given_wrong_side_should_return_empty, "this ++is not a matching phrase", Vec::<KarmaCapture>::new()),
        (given_no_space_should_return_empty, "this++is not a matching phrase", Vec::<KarmaCapture>::new()),
        (given_wrong_side_at_start_should_return_empty, "++this is not a matching phrase", Vec::<KarmaCapture>::new()),
        (given_preformatted_text_should_return_empty, "`preformatted++ to the max`", Vec::<KarmaCapture>::new()),
        (given_preformatted_text_should_return_empty_2, "`preformatted`++ to the max", Vec::<KarmaCapture>::new()),
        (given_preformatted_text_should_return_empty_3, "`preformatte`d++ to the max", Vec::<KarmaCapture>::new()),
        (given_preformatted_multiline_text_should_return_empty, "```\nlet var = 0\nvar++\n```", Vec::<KarmaCapture>::new())
    }
}
