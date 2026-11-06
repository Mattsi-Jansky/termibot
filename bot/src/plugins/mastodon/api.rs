use crate::plugins::mastodon::link::PostLink;
use crate::plugins::mastodon::post::Post;
use crate::plugins::mastodon::public_address::ensure_public;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use std::fmt;
use url::Url;

const MAX_POST_BYTES: usize = 1024 * 1024;
const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug)]
pub struct FetchError(pub String);

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> Self {
        FetchError(format!("HTTP error: {err}"))
    }
}

impl From<serde_json::Error> for FetchError {
    fn from(err: serde_json::Error) -> Self {
        FetchError(format!("Not a Mastodon post: {err}"))
    }
}

impl From<std::io::Error> for FetchError {
    fn from(err: std::io::Error) -> Self {
        FetchError(format!("Couldn't look up address: {err}"))
    }
}

impl From<url::ParseError> for FetchError {
    fn from(err: url::ParseError) -> Self {
        FetchError(format!("Invalid URL: {err}"))
    }
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait MastodonApi: Send + Sync {
    async fn fetch_post(&self, link: &PostLink) -> Result<Post, FetchError>;
    async fn download_image(&self, url: &str) -> Result<Vec<u8>, FetchError>;
}

pub struct HttpMastodonApi {
    http: reqwest::Client,
}

impl HttpMastodonApi {
    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }

    async fn get(&self, url: &str, max_bytes: usize) -> Result<Vec<u8>, FetchError> {
        let url = Url::parse(url)?;
        ensure_public(&url).await?;
        let mut response = self
            .http
            .get(url.clone())
            .send()
            .await?
            .error_for_status()?;
        if response.url() != &url {
            ensure_public(response.url()).await?;
        }

        read_body(&mut response, max_bytes).await
    }
}

#[async_trait]
impl MastodonApi for HttpMastodonApi {
    async fn fetch_post(&self, link: &PostLink) -> Result<Post, FetchError> {
        let body = self.get(&link.api_url(), MAX_POST_BYTES).await?;
        Ok(serde_json::from_slice(&body)?)
    }

    async fn download_image(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.get(url, MAX_IMAGE_BYTES).await
    }
}

async fn read_body(
    response: &mut reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, FetchError> {
    let too_large = || FetchError(format!("Response is larger than {max_bytes} bytes"));
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(too_large());
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len() + chunk.len() > max_bytes {
            return Err(too_large());
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn serve_once(response: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0; 1024];
            let _ = socket.read(&mut request).await;
            socket.write_all(response.as_bytes()).await.unwrap();
        });
        format!("http://{address}/")
    }

    async fn read_from(response: &'static str, max_bytes: usize) -> Result<Vec<u8>, FetchError> {
        let url = serve_once(response).await;
        let mut response = reqwest::get(url).await.unwrap();
        read_body(&mut response, max_bytes).await
    }

    #[tokio::test]
    async fn should_read_body_within_limit() {
        let body = read_from("HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello", 10).await;

        assert_eq!(body.unwrap(), b"hello");
    }

    #[tokio::test]
    async fn should_refuse_body_declared_larger_than_limit() {
        let body = read_from(
            "HTTP/1.1 200 OK\r\nContent-Length: 20\r\n\r\n01234567890123456789",
            10,
        )
        .await;

        assert!(body.is_err());
    }

    #[tokio::test]
    async fn should_refuse_body_larger_than_limit_without_declared_length() {
        let body = read_from(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n14\r\n01234567890123456789\r\n0\r\n\r\n",
            10,
        )
        .await;

        assert!(body.is_err());
    }
}
