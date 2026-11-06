use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UploadUrlResponse {
    pub ok: bool,
    pub error: Option<String>,
    pub upload_url: Option<String>,
    pub file_id: Option<String>,
}
