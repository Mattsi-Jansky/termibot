mod test_config;

use std::fs;
use std::path::{Path};
use glob::{glob};
use regex::{Regex, Replacer};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use rvcr::{VCRMiddleware, VCRMode};
use framework::SlackClient;
use crate::test_config::TEST_CONFIG;

const FAKE_TOKEN: &str = "xoxn-not-a-real-token";

#[tokio::test]
async fn should_send_messages_to_channels_and_threads() {
    let builder = TestClientBuilder::new("should_send_messages_to_channels_and_threads");
    let client = builder.new_client();
    let result = client.message_channel("#bots", "foobar").await;
    assert!(result.is_ok());
    let result = result.unwrap();

    let new_message = &format!("replying to {}", result.message.text)[..];
    let result2 = client.message_thread("#bots", &result.message, new_message);
    assert!(result2.await.is_ok());
}

struct TestClientBuilder {
    name: String
}

impl Drop for TestClientBuilder {
    fn drop(&mut self) {
        //On drop remove any secure credentials
        let regex = Regex::new(r"(xoxb|xapp-1|xoxp|xoxa-2|xoxr)-([a-zA-Z0-9]+-?){3}").unwrap();
        let files = glob(&format!("{}/tests/resources/*", env!("CARGO_MANIFEST_DIR"))[..]).unwrap();
        for file in files {
            let file = file.unwrap();
            let path = file.as_path();
            let contents = fs::read_to_string(path).expect("Cleaning cassette failed - DO NOT COMMIT!");
            let cleaned_contents = regex.replace_all(&contents, FAKE_TOKEN).to_string();
            fs::write(path, cleaned_contents).expect("Writing cleaned cassette failed - DO NOT COMMIT!");
        }
    }
}

impl TestClientBuilder {
    fn new(name: &str) -> TestClientBuilder {
        TestClientBuilder { name: name.to_string() }
    }

    fn new_client(&self) -> SlackClient {
        let path = format!("{}/tests/resources/{}.vcr.json", env!("CARGO_MANIFEST_DIR"), self.name);
        let path = Path::new(&path);
        if path.exists() {
            fs::remove_file(path).expect(&format!("Failed to delete old cassette {:?}", path)[..]);
        }

        let mut middleware = VCRMiddleware::try_from(path.to_path_buf())
            .unwrap();
        if TEST_CONFIG.is_record_mode {
            middleware = middleware.with_mode(VCRMode::Record);
        }

        let vcr_client: ClientWithMiddleware = ClientBuilder::new(reqwest::Client::new())
            .with(middleware)
            .build();

        let client = SlackClient::with_client(&TEST_CONFIG.bot_token[..], vcr_client);
        client
    }
}