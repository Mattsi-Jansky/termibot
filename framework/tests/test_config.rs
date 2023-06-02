use config_file::FromConfigFile;

use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TestConfig {
    pub bot_token: String,
}

lazy_static! {
    pub static ref TEST_CONFIG: TestConfig = TestConfig::from_config_file("config/config.toml").unwrap();
}
