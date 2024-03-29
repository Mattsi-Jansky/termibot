use crate::error::SlackClientError;
use crate::models::blocks::Block;

#[derive(Debug, PartialEq)]
pub struct MessageBody {
    text: Option<String>,
    blocks: Vec<Block>, //attachments
}

impl MessageBody {
    pub fn new(blocks: Vec<Block>, text: Option<String>) -> Result<Self, SlackClientError> {
        if blocks.is_empty() && text.is_none() {
            Err(SlackClientError("Must have..".to_string()))
        } else {
            Ok(Self { blocks, text })
        }
    }

    pub fn from_text(text: &str) -> Self {
        Self {
            text: Some(text.to_string()),
            blocks: vec![],
        }
    }

    pub fn get_text(&self) -> String {
        self.text.clone().unwrap_or_default()
    }

    pub fn get_blocks(&self) -> &Vec<Block> {
        &self.blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::blocks::elements::text::RichTextSectionElement;
    use crate::models::blocks::elements::BlockElement;
    use crate::models::blocks::objects::text::TextBody;
    use crate::models::blocks::text::RichTextBlock;

    #[test]
    fn given_no_text_or_attachments_constructor_should_return_error() {
        let result = MessageBody::new(vec![], None);

        assert!(result.is_err())
    }

    #[test]
    fn given_text_but_no_blocks_constructor_should_succeed() {
        let result = MessageBody::new(vec![], Some("test".to_string()));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.text, Some("test".to_string()));
        assert_eq!(result.blocks.len(), 0);
    }

    #[test]
    fn given_blocks_but_no_text_constructor_should_succeed() {
        let result = MessageBody::new(
            vec![Block::RichText(
                RichTextBlock::new()
                    .elements(vec![BlockElement::RichTextSection(
                        RichTextSectionElement::new()
                            .elements(vec![BlockElement::Text(
                                TextBody::new().text("test".to_string()).build(),
                            )])
                            .build(),
                    )])
                    .build(),
            )],
            None,
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.text, None);
        assert_eq!(result.blocks.len(), 1);
    }

    #[test]
    fn given_both_blocks_and_text_constructor_should_succeed() {
        let result = MessageBody::new(
            vec![Block::RichText(
                RichTextBlock::new()
                    .elements(vec![BlockElement::RichTextSection(
                        RichTextSectionElement::new()
                            .elements(vec![BlockElement::Text(
                                TextBody::new().text("test".to_string()).build(),
                            )])
                            .build(),
                    )])
                    .build(),
            )],
            Some("test".to_string()),
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.text, Some("test".to_string()));
        assert_eq!(result.blocks.len(), 1);
    }
}
