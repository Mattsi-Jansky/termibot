use termibot::plugins::songlink::SongLinkPlugin;
use termibot::SlackBot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("slack_morphism=debug")
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let bot = SlackBot::default().with::<SongLinkPlugin>();
    bot.run().await?;

    Ok(())
}
