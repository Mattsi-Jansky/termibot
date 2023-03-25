use config_file::FromConfigFile;

use lazy_static::lazy_static;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub app_token: String,
    pub bot_token: String
}

lazy_static! {
    pub static ref CONFIG: Config = Config::from_config_file("config/config.toml").unwrap();
}