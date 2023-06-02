mod test_config;

use framework::SlackClient;
use crate::test_config::TEST_CONFIG;

#[tokio::test]
async fn test() {
    let client = SlackClient::new(&TEST_CONFIG.bot_token[..]);
    // let result = client.send_thread_reply().await.unwrap();
    let result = client.message_channel("#bots", "foobar").await.unwrap();
    println!("====== WAT: {:#?}", result);

    let new_message = &format!("replying to {}", result.message.text)[..];
    let result2 = client.message_thread("#bots", &result.message, new_message);
    println!("====== WAT: {:#?}", result2.await);
}
