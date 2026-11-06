use std::fmt;

#[derive(PartialEq)]
pub struct FileUpload {
    pub filename: String,
    pub content: Vec<u8>,
    pub alt_text: Option<String>,
}

impl FileUpload {
    pub fn new(filename: &str, content: Vec<u8>) -> Self {
        Self {
            filename: filename.to_string(),
            content,
            alt_text: None,
        }
    }

    pub fn with_alt_text(mut self, alt_text: &str) -> Self {
        self.alt_text = Some(alt_text.to_string());
        self
    }
}

impl fmt::Debug for FileUpload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileUpload")
            .field("filename", &self.filename)
            .field("content", &format_args!("{} bytes", self.content.len()))
            .field("alt_text", &self.alt_text)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_shows_size_of_content_instead_of_bytes() {
        let upload = FileUpload::new("cat.png", vec![7; 1024]);

        let debug = format!("{upload:?}");

        assert!(debug.contains("1024 bytes"), "{debug}");
        assert!(!debug.contains("7, 7"), "{debug}");
    }
}
