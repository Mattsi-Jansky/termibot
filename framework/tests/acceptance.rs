mod test_config;

use framework::SlackClient;
use crate::test_config::TEST_CONFIG;

#[tokio::test]
async fn test() {
    let client = SlackClient::new(&TEST_CONFIG.bot_token[..]);
    let result = client.send_thread_reply().await.unwrap();

    println!("====== WAT: {:?}", result.text().await);
}
