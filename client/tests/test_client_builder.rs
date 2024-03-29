use client::rate_limiter::RateLimitingMiddleware;
use client::ReqwestSlackClient;
use config_file::FromConfigFile;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rvcr::{VCRMiddleware, VCRMode};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct TestConfig {
    pub bot_token: String,
    pub app_token: String,
    pub is_record_mode: bool,
}

lazy_static! {
    pub static ref TEST_CONFIG: TestConfig = TestConfig::from_config_file("config/config.toml")
        .unwrap_or(TestConfig {
            bot_token: String::from(FAKE_TOKEN),
            app_token: String::from(FAKE_TOKEN),
            is_record_mode: false
        });
    pub static ref TOKEN_REGEX: Regex =
        Regex::new(r"(xoxb|xapp-1|xoxp|xoxa-2|xoxr)-([a-zA-Z0-9]+-?){3}").unwrap();
    pub static ref WSS_URL_REGEX: Regex = Regex::new("\\\\\"url\\\\\":\\\\\".*?\\\\\"").unwrap();
}
const FAKE_TOKEN: &str = "xoxn-not-a-real-token";
const FAKE_WSS_URL: &str = "\\\"url\\\":\\\"ws://localhost:12345/\\\"";

pub struct TestClientBuilder {
    name: String,
}

impl Drop for TestClientBuilder {
    fn drop(&mut self) {
        //On drop, if we have recorded a file, remove any secure credentials
        if TEST_CONFIG.is_record_mode {
            let files =
                glob::glob(&format!("{}/tests/resources/*", env!("CARGO_MANIFEST_DIR"))[..])
                    .unwrap();
            for file in files {
                let file = file.expect("Cleaning cassette failed - DO NOT COMMIT!");
                let path = file.as_path();
                let contents =
                    fs::read_to_string(path).expect("Cleaning cassette failed - DO NOT COMMIT!");
                let cleaned_contents = TOKEN_REGEX.replace_all(&contents, FAKE_TOKEN).to_string();
                let cleaned_contents = WSS_URL_REGEX
                    .replace_all(&cleaned_contents, FAKE_WSS_URL)
                    .to_string();
                fs::write(path, cleaned_contents)
                    .expect("Writing cleaned cassette failed - DO NOT COMMIT!");
            }
        }
    }
}

impl TestClientBuilder {
    pub fn new(name: &str) -> TestClientBuilder {
        TestClientBuilder {
            name: name.to_string(),
        }
    }

    pub fn new_client(&self) -> ReqwestSlackClient {
        let path = format!(
            "{}/tests/resources/{}.vcr.json",
            env!("CARGO_MANIFEST_DIR"),
            self.name
        );
        let path = Path::new(&path);
        if TEST_CONFIG.is_record_mode && path.exists() {
            fs::remove_file(path).expect(&format!("Failed to delete old cassette {:?}", path)[..]);
        }

        let mut vcr_testing_middleware = VCRMiddleware::try_from(path.to_path_buf()).unwrap();
        if TEST_CONFIG.is_record_mode {
            vcr_testing_middleware = vcr_testing_middleware.with_mode(VCRMode::Record);
        }

        let vcr_client: ClientWithMiddleware = ClientBuilder::new(reqwest::Client::new())
            .with(RateLimitingMiddleware::new())
            .with(vcr_testing_middleware)
            .build();

        if TEST_CONFIG.is_record_mode {
            ReqwestSlackClient::with_client(
                &TEST_CONFIG.bot_token[..],
                &TEST_CONFIG.app_token[..],
                vcr_client,
            )
        } else {
            ReqwestSlackClient::with_client(FAKE_TOKEN, FAKE_TOKEN, vcr_client)
        }
    }
}
