use lazy_static::lazy_static;
use serde::Deserialize;
use config_file::FromConfigFile;
use framework::SlackBot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    activate_logging();
    SlackBot::new(&CONFIG.bot_token[..], &CONFIG.app_token[..]).run().await.unwrap();

    // SlackBot::default().with::<SongLinkPlugin>().run().await?;

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
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("slack_morphism=debug")
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}