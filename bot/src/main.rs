use termibot::plugins::songlink::SongLinkPlugin;
use termibot::SlackBot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    SlackBot::default().with::<SongLinkPlugin>().run().await?;

    Ok(())
}
