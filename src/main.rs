use termibot::modules::songlink::SongLinkModule;
use termibot::SlackBot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("slack_morphism=debug")
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let bot = SlackBot { module: SongLinkModule {} };
    bot.run().await?;

    Ok(())
}
