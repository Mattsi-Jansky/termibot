use config_file::FromConfigFile;

use lazy_static::lazy_static;
use serde::Deserialize;
use slack_morphism::SlackApiTokenValue;

#[derive(Deserialize)]
pub struct Config {
    pub app_token: SlackApiTokenValue,
    pub bot_token: SlackApiTokenValue,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::from_config_file("config/config.toml").unwrap();
}
