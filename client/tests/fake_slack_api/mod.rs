use client::ReqwestSlackClient;
use reqwest::{Request, Response};
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard};
use task_local_extensions::Extensions;

pub struct RecordedRequest {
    pub url: String,
    pub body: Vec<u8>,
}

impl RecordedRequest {
    pub fn json(&self) -> Value {
        serde_json::from_slice(&self.body).expect("request body should be JSON")
    }

    pub fn form(&self) -> HashMap<String, String> {
        url::form_urlencoded::parse(&self.body)
            .into_owned()
            .collect()
    }
}

#[derive(Clone, Default)]
pub struct FakeSlackApi {
    responses: Arc<Mutex<VecDeque<(u16, String)>>>,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

impl FakeSlackApi {
    pub fn then_respond(self, status: u16, body: &str) -> Self {
        self.responses
            .lock()
            .unwrap()
            .push_back((status, body.to_string()));
        self
    }

    pub fn client(&self) -> ReqwestSlackClient {
        let http = ClientBuilder::new(reqwest::Client::new())
            .with(self.clone())
            .build();
        ReqwestSlackClient::with_client("xoxb-fake-token", "xapp-fake-token", http)
    }

    pub fn requests(&self) -> MutexGuard<'_, Vec<RecordedRequest>> {
        self.requests.lock().unwrap()
    }
}

#[async_trait::async_trait]
impl Middleware for FakeSlackApi {
    async fn handle(
        &self,
        request: Request,
        _extensions: &mut Extensions,
        _next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let body = request
            .body()
            .and_then(|body| body.as_bytes())
            .unwrap_or_default()
            .to_vec();
        self.requests.lock().unwrap().push(RecordedRequest {
            url: request.url().to_string(),
            body,
        });

        let (status, body) = self
            .responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("FakeSlackApi has no response left for this request");
        Ok(http::Response::builder()
            .status(status)
            .body(body)
            .unwrap()
            .into())
    }
}
