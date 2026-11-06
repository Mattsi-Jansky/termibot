use crate::actions::{Action, Image};
use async_trait::async_trait;
use client::error::SlackClientError;
use client::models::blocks::image::ImageBlock;
use client::models::blocks::Block;
use client::models::file_upload::FileUpload;
use client::models::message_body::MessageBody;
use client::models::message_id::MessageId;
use client::SlackClient;
use mockall::automock;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

const UPLOAD_PROCESSING_RETRY_DELAYS: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(2),
    Duration::from_secs(3),
];

#[async_trait]
#[automock]
pub trait ActionHandler {
    async fn handle(
        &self,
        action: Action,
        client: Arc<dyn SlackClient + Send + Sync>,
    ) -> Result<(), SlackClientError>;
}

#[derive(Default)]
pub struct DefaultActionHandler {}

#[async_trait]
impl ActionHandler for DefaultActionHandler {
    async fn handle(
        &self,
        action: Action,
        client: Arc<dyn SlackClient + Send + Sync>,
    ) -> Result<(), SlackClientError> {
        match action {
            Action::MessageChannel { channel, message } => client
                .message_channel(&channel[..], &message)
                .await
                .map(|_| ())?,
            Action::ReplyToThread {
                channel,
                thread_id,
                message,
            } => client
                .message_thread(&channel, &thread_id, &message)
                .await
                .map(|_| ())?,
            Action::ReplyToThreadWithImages {
                channel,
                thread_id,
                mut message,
                images,
            } => {
                let mut image_blocks = Vec::with_capacity(images.len());
                for image in images {
                    image_blocks.push(image_block(image, client.as_ref()).await);
                }
                message.append_blocks(image_blocks);
                reply_once_uploads_are_processed(client.as_ref(), &channel, &thread_id, &message)
                    .await?
            }
            Action::FirstSuccessful(actions) => {
                self.handle_first_successful(actions, client).await?
            }
            Action::InOrder(actions) => self.handle_in_order(actions, client).await?,
        }

        Ok(())
    }
}

impl DefaultActionHandler {
    async fn handle_first_successful(
        &self,
        actions: Vec<Action>,
        client: Arc<dyn SlackClient + Send + Sync>,
    ) -> Result<(), SlackClientError> {
        let mut last_error = SlackClientError("There were no actions to try".to_string());
        for action in actions {
            match self.handle(action, client.clone()).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    warn!("Action failed, trying the next one: {:?}", err);
                    last_error = err;
                }
            }
        }

        Err(last_error)
    }

    async fn handle_in_order(
        &self,
        actions: Vec<Action>,
        client: Arc<dyn SlackClient + Send + Sync>,
    ) -> Result<(), SlackClientError> {
        let mut result = Ok(());
        for action in actions {
            if let Err(err) = self.handle(action, client.clone()).await {
                warn!("Action failed, carrying on with the next one: {:?}", err);
                result = Err(err);
            }
        }

        result
    }
}

async fn image_block(image: Image, client: &(dyn SlackClient + Send + Sync)) -> Block {
    match image.content {
        Some(content) => {
            let upload = FileUpload::new(filename_from_url(&image.url), content)
                .with_alt_text(&image.alt_text);
            match client.upload_file(upload).await {
                Ok(file_id) => ImageBlock::from_slack_file(&file_id, &image.alt_text),
                Err(err) => {
                    warn!(
                        "Couldn't upload {}, linking to it instead: {:?}",
                        image.url, err
                    );
                    ImageBlock::from_url(&image.url, &image.alt_text)
                }
            }
        }
        None => ImageBlock::from_url(&image.url, &image.alt_text),
    }
    .into()
}

async fn reply_once_uploads_are_processed(
    client: &(dyn SlackClient + Send + Sync),
    channel: &str,
    thread_id: &MessageId,
    message: &MessageBody,
) -> Result<(), SlackClientError> {
    let mut result = client.message_thread(channel, thread_id, message).await;
    for delay in UPLOAD_PROCESSING_RETRY_DELAYS {
        match &result {
            Err(err) if is_upload_still_processing(err) => {
                info!("Slack is still processing an uploaded image, trying again in {delay:?}");
                tokio::time::sleep(delay).await;
                result = client.message_thread(channel, thread_id, message).await;
            }
            _ => break,
        }
    }

    result.map(|_| ())
}

/// Slack calls a freshly uploaded file invalid until it has finished processing it
fn is_upload_still_processing(error: &SlackClientError) -> bool {
    error.0.contains("invalid slack file")
}

fn filename_from_url(url: &str) -> &str {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("image")
}

#[cfg(test)]
mod tests {
    use super::*;
    use client::models::auth_test_response::AuthTestResponse;
    use client::models::blocks::section::SectionBlock;
    use client::models::http_response::HttpApiResponse;
    use client::models::http_response::Message;
    use client::models::message_body::MessageBody;
    use client::models::message_id::MessageId;
    use client::MockSlackClient;
    use mockall::Sequence;

    #[tokio::test]
    async fn given_channel_message_action_should_send_message_to_channel() {
        let handler = DefaultActionHandler::default();
        let test_action = Action::MessageChannel {
            channel: String::from("#bots"),
            message: MessageBody::from_text("hello world"),
        };
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_message_channel()
            .withf(|channel, message| channel == "#bots" && message.get_text() == "hello world")
            .times(1)
            .returning(|_, _| {
                Ok(HttpApiResponse {
                    ok: true,
                    message: Some(Message {
                        id: "".to_string().into(),
                        text: "".to_string(),
                        user: "".to_string(),
                    }),
                    error: None,
                    errors: None,
                })
            });

        handler
            .handle(test_action, Arc::new(mock_client))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn given_thread_reply_should_send_message_to_thread() {
        let handler = DefaultActionHandler::default();
        let test_action = Action::ReplyToThread {
            channel: String::from("#bots"),
            thread_id: "thread-id".to_string().into(),
            message: MessageBody::from_text("hello world"),
        };
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_message_thread()
            .withf(|channel, thread, message| {
                channel == "#bots"
                    && thread == &MessageId::new(String::from("thread-id"))
                    && message.get_text() == "hello world"
            })
            .times(1)
            .returning(|_, _, _| {
                Ok(HttpApiResponse {
                    ok: true,
                    message: Some(Message {
                        id: "".to_string().into(),
                        text: "".to_string(),
                        user: "".to_string(),
                    }),
                    error: None,
                    errors: None,
                })
            });

        handler
            .handle(test_action, Arc::new(mock_client))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn first_successful_stops_at_the_first_action_that_succeeds() {
        let mut mock_client = MockSlackClient::new();
        expect_channel_message(&mut mock_client, "first", 1, || Ok(sent()));
        expect_channel_message(&mut mock_client, "second", 0, || Ok(sent()));

        let result = DefaultActionHandler::default()
            .handle(
                Action::FirstSuccessful(vec![channel_message("first"), channel_message("second")]),
                Arc::new(mock_client),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn first_successful_tries_the_next_action_when_one_fails() {
        let mut mock_client = MockSlackClient::new();
        expect_channel_message(&mut mock_client, "first", 1, || {
            Err(failure("first failed"))
        });
        expect_channel_message(&mut mock_client, "second", 1, || Ok(sent()));

        let result = DefaultActionHandler::default()
            .handle(
                Action::FirstSuccessful(vec![channel_message("first"), channel_message("second")]),
                Arc::new(mock_client),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn first_successful_fails_with_the_last_error_when_every_action_fails() {
        let mut mock_client = MockSlackClient::new();
        expect_channel_message(&mut mock_client, "first", 1, || {
            Err(failure("first failed"))
        });
        expect_channel_message(&mut mock_client, "second", 1, || {
            Err(failure("second failed"))
        });

        let result = DefaultActionHandler::default()
            .handle(
                Action::FirstSuccessful(vec![channel_message("first"), channel_message("second")]),
                Arc::new(mock_client),
            )
            .await;

        assert_eq!(result.unwrap_err().0, "second failed");
    }

    #[tokio::test]
    async fn images_with_content_are_uploaded_and_shown_from_slack() {
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_upload_file()
            .withf(|upload| {
                upload == &FileUpload::new("cat.png", b"cat pixels".to_vec()).with_alt_text("A cat")
            })
            .times(1)
            .returning(|_| Ok("F123".to_string()));
        expect_thread_reply_with_blocks(
            &mut mock_client,
            vec![
                SectionBlock::new_markdown("Look"),
                ImageBlock::from_slack_file("F123", "A cat").into(),
            ],
        );

        DefaultActionHandler::default()
            .handle(
                reply_with_images(vec![
                    Image::new(CAT_URL, "A cat").with_content(b"cat pixels".to_vec())
                ]),
                Arc::new(mock_client),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn images_are_linked_when_their_upload_fails() {
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_upload_file()
            .times(1)
            .returning(|_| Err(failure("upload failed")));
        expect_thread_reply_with_blocks(
            &mut mock_client,
            vec![
                SectionBlock::new_markdown("Look"),
                ImageBlock::from_url(CAT_URL, "A cat").into(),
            ],
        );

        DefaultActionHandler::default()
            .handle(
                reply_with_images(vec![
                    Image::new(CAT_URL, "A cat").with_content(b"cat pixels".to_vec())
                ]),
                Arc::new(mock_client),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn images_without_content_are_linked_without_uploading() {
        let mut mock_client = MockSlackClient::new();
        mock_client.expect_upload_file().times(0);
        expect_thread_reply_with_blocks(
            &mut mock_client,
            vec![
                SectionBlock::new_markdown("Look"),
                ImageBlock::from_url(CAT_URL, "A cat").into(),
            ],
        );

        DefaultActionHandler::default()
            .handle(
                reply_with_images(vec![Image::new(CAT_URL, "A cat")]),
                Arc::new(mock_client),
            )
            .await
            .unwrap();
    }

    #[test]
    fn filename_is_the_last_part_of_the_url_path() {
        assert_eq!(
            filename_from_url("https://example.com/a/cat.png?v=1#top"),
            "cat.png"
        );
        assert_eq!(filename_from_url("https://example.com/"), "image");
    }

    #[test]
    fn image_debug_output_shows_size_of_content_instead_of_bytes() {
        let image = Image::new(CAT_URL, "A cat").with_content(vec![7; 1024]);

        let debug = format!("{image:?}");

        assert!(debug.contains("1024 bytes"), "{debug}");
        assert!(!debug.contains("7, 7"), "{debug}");
    }

    #[tokio::test(start_paused = true)]
    async fn replies_with_uploaded_images_are_retried_while_slack_processes_the_upload() {
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_upload_file()
            .returning(|_| Ok("F123".to_string()));
        let mut sequence = Sequence::new();
        mock_client
            .expect_message_thread()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_, _, _| Err(failure(UPLOAD_STILL_PROCESSING)));
        mock_client
            .expect_message_thread()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_, _, _| Ok(sent()));

        let result = DefaultActionHandler::default()
            .handle(
                reply_with_images(vec![cat_with_content()]),
                Arc::new(mock_client),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn replies_with_uploaded_images_give_up_if_slack_never_finishes_processing() {
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_upload_file()
            .returning(|_| Ok("F123".to_string()));
        mock_client
            .expect_message_thread()
            .times(UPLOAD_PROCESSING_RETRY_DELAYS.len() + 1)
            .returning(|_, _, _| Err(failure(UPLOAD_STILL_PROCESSING)));

        let result = DefaultActionHandler::default()
            .handle(
                reply_with_images(vec![cat_with_content()]),
                Arc::new(mock_client),
            )
            .await;

        assert_eq!(result.unwrap_err().0, UPLOAD_STILL_PROCESSING);
    }

    #[tokio::test(start_paused = true)]
    async fn replies_with_images_are_not_retried_for_other_errors() {
        let mut mock_client = MockSlackClient::new();
        mock_client
            .expect_message_thread()
            .times(1)
            .returning(|_, _, _| Err(failure("invalid_blocks: [downloading image failed]")));

        let result = DefaultActionHandler::default()
            .handle(
                reply_with_images(vec![Image::new(CAT_URL, "A cat")]),
                Arc::new(mock_client),
            )
            .await;

        assert!(result.is_err());
    }

    const CAT_URL: &str = "https://files.example.com/media/cat.png?v=1";
    const UPLOAD_STILL_PROCESSING: &str =
        "invalid_blocks: [invalid slack file [json-pointer:/blocks/1/slack_file.id/slack_file]]";

    fn cat_with_content() -> Image {
        Image::new(CAT_URL, "A cat").with_content(b"cat pixels".to_vec())
    }

    fn reply_with_images(images: Vec<Image>) -> Action {
        Action::ReplyToThreadWithImages {
            channel: String::from("#bots"),
            thread_id: "thread-id".to_string().into(),
            message: MessageBody::new(vec![SectionBlock::new_markdown("Look")], None).unwrap(),
            images,
        }
    }

    fn expect_thread_reply_with_blocks(mock_client: &mut MockSlackClient, blocks: Vec<Block>) {
        mock_client
            .expect_message_thread()
            .withf(move |_, _, message| message.get_blocks() == &blocks)
            .times(1)
            .returning(|_, _, _| Ok(sent()));
    }

    #[tokio::test(start_paused = true)]
    async fn in_order_runs_each_action_once_the_one_before_has_finished() {
        let client = Arc::new(SlowFirstMessageClient::default());

        DefaultActionHandler::default()
            .handle(
                Action::InOrder(vec![channel_message("first"), channel_message("second")]),
                client.clone(),
            )
            .await
            .unwrap();

        assert_eq!(
            *client.events.lock().unwrap(),
            [
                "started first",
                "finished first",
                "started second",
                "finished second"
            ]
        );
    }

    #[derive(Default)]
    struct SlowFirstMessageClient {
        events: std::sync::Mutex<Vec<String>>,
    }

    #[async_trait]
    impl SlackClient for SlowFirstMessageClient {
        async fn message_channel(
            &self,
            _channel: &str,
            message: &MessageBody,
        ) -> Result<HttpApiResponse, SlackClientError> {
            let text = message.get_text();
            self.events.lock().unwrap().push(format!("started {text}"));
            if text == "first" {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            self.events.lock().unwrap().push(format!("finished {text}"));
            Ok(sent())
        }

        async fn get_identity(&self) -> Result<AuthTestResponse, SlackClientError> {
            unimplemented!()
        }

        async fn message_thread(
            &self,
            _channel: &str,
            _parent: &MessageId,
            _message: &MessageBody,
        ) -> Result<HttpApiResponse, SlackClientError> {
            unimplemented!()
        }

        async fn get_websocket_url(&self) -> Result<url::Url, SlackClientError> {
            unimplemented!()
        }

        async fn upload_file(&self, _file: FileUpload) -> Result<String, SlackClientError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn in_order_carries_on_after_a_failure_and_reports_it() {
        let mut mock_client = MockSlackClient::new();
        expect_channel_message(&mut mock_client, "first", 1, || {
            Err(failure("first failed"))
        });
        expect_channel_message(&mut mock_client, "second", 1, || Ok(sent()));

        let result = DefaultActionHandler::default()
            .handle(
                Action::InOrder(vec![channel_message("first"), channel_message("second")]),
                Arc::new(mock_client),
            )
            .await;

        assert_eq!(result.unwrap_err().0, "first failed");
    }

    fn channel_message(text: &str) -> Action {
        Action::MessageChannel {
            channel: String::from("#bots"),
            message: MessageBody::from_text(text),
        }
    }

    fn expect_channel_message(
        mock_client: &mut MockSlackClient,
        text: &'static str,
        times: usize,
        result: fn() -> Result<HttpApiResponse, SlackClientError>,
    ) {
        mock_client
            .expect_message_channel()
            .withf(move |_, message| message.get_text() == text)
            .times(times)
            .returning(move |_, _| result());
    }

    fn sent() -> HttpApiResponse {
        HttpApiResponse {
            ok: true,
            message: None,
            error: None,
            errors: None,
        }
    }

    fn failure(error: &str) -> SlackClientError {
        SlackClientError(error.to_string())
    }
}
