use framework::send_thread_reply;

#[tokio::test]
async fn test() {
    let result = send_thread_reply().await.unwrap();

    println!("====== WAT: {:?}", result.text().await);
    // println!("====== WOT: {:?}", result.headers());
    // println!("====== STA: {:?}", result.status());
}