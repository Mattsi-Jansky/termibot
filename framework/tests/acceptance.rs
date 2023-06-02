use framework::SlackClient;

const BOT_TOKEN: &str = "fill_here";

#[tokio::test]
async fn test() {
    let client = SlackClient::new(BOT_TOKEN);
    let result = client.send_thread_reply().await.unwrap();

    println!("====== WAT: {:?}", result.text().await);
}
