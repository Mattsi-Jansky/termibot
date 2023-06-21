use crate::error::SlackClientError;
use crate::models::blocks::Block;

pub struct MessageTemplate {
    text: Option<String>,
    blocks: Vec<Block>
    //attachments
}

impl MessageTemplate {
    pub fn new(blocks: Vec<Block>, text: Option<String>) -> Result<Self, SlackClientError>{
        if blocks.is_empty() && text.is_none() {
            Err(SlackClientError("Must have..".to_string()))
        } else {
            Ok(Self {blocks, text})
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::blocks::elements::BlockElement;
    use crate::models::blocks::elements::text::{RichTextSectionElement, TextElement};
    use crate::models::blocks::text::RichTextBlock;
    use super::*;

    #[test]
    fn given_no_text_or_attachments_constructor_should_return_error() {
        let result = MessageTemplate::new(vec![], None);

        assert!(result.is_err())
    }

    #[test]
    fn given_text_but_no_blocks_constructor_should_succeed() {
        let result = MessageTemplate::new(vec![], Some("test".to_string()));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.text, Some("test".to_string()));
        assert_eq!(result.blocks.len(), 0);
    }

    #[test]
    fn given_blocks_but_no_text_constructor_should_succeed() {
        let result = MessageTemplate::new(vec![
            Block::RichText(RichTextBlock::new()
                .elements(vec![BlockElement::RichTextSection(
                    RichTextSectionElement::new()
                        .elements(vec![BlockElement::Text(
                            TextElement::new().text("test".to_string()).build()
                        )])
                        .build()
                )]).build())
        ], None);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.text,None);
        assert_eq!(result.blocks.len(), 1);
    }

    #[test]
    fn given_both_blocks_and_text_constructor_should_succeed() {
        let result = MessageTemplate::new(vec![
                                            Block::RichText(RichTextBlock::new()
                                              .elements(vec![BlockElement::RichTextSection(
                                                  RichTextSectionElement::new()
                                                      .elements(vec![BlockElement::Text(
                                                          TextElement::new().text("test".to_string()).build()
                                                      )])
                                                      .build()
                                              )]).build())
        ], Some("test".to_string()));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.text, Some("test".to_string()));
        assert_eq!(result.blocks.len(), 1);
    }
}
