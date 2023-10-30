mod plugins;

use crate::plugins::emoji_changelog::EmojiChangelogPlugin;
use crate::plugins::songlink::SongLinkPlugin;
use config_file::FromConfigFile;
use framework::SlackBot;
use karma::services::karma_repository::{KarmaRepository, SqliteKarmaRepository};
use karma::KarmaPlugin;
use lazy_static::lazy_static;
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    activate_logging().unwrap();
    SlackBot::new(&CONFIG.bot_token[..], &CONFIG.app_token[..])
        .with_plugin(Box::new(SongLinkPlugin {}))
        .with_plugin(Box::new(EmojiChangelogPlugin::new("#general".to_string())))
        .with_dyn_service::<dyn KarmaRepository + Send + Sync>(Box::new(
            SqliteKarmaRepository::default().await,
        ))
        .with_plugin(Box::new(KarmaPlugin::default()))
        .run()
        .await
        .unwrap();

    Ok(())
}

#[derive(Deserialize)]
pub struct Config {
    pub app_token: String,
    pub bot_token: String,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::from_config_file("config/config.toml").unwrap();
}

fn activate_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
