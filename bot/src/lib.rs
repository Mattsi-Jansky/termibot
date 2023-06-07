// use slack_morphism::prelude::*;
//
// use self::core::*;
// use crate::actions::handlers::{ActionHandler, DefaultActionHandler};
// use crate::config::CONFIG;
// use crate::plugins::Plugin;
// use hyper::client::HttpConnector;
// use hyper_rustls::HttpsConnector;
// use std::sync::Arc;
//
// mod actions;
// mod config;
// mod core;
// pub mod plugins;
//
// pub struct SlackBot {
//     pub plugins: Vec<Box<dyn Plugin + Send + Sync>>,
//     pub action_handler: Box<dyn ActionHandler + Send + Sync>,
// }
//
// impl Default for SlackBot {
//     fn default() -> Self {
//         Self {
//             plugins: vec![],
//             action_handler: Box::new(DefaultActionHandler::new()),
//         }
//     }
// }
//
// impl SlackBot {
//     pub fn with<T: Plugin + Send + Sync + 'static>(mut self) -> Self {
//         self.plugins.push(Box::new(T::new()));
//         self
//     }
//
//     pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//         Self::activate_logging()?;
//         let client = Arc::new(SlackClient::new(SlackClientHyperConnector::new()));
//         let listener = Self::build_listener(self.build_listener_environment(client));
//
//         listener.listen_for(&Self::get_app_token()).await?;
//         listener.serve().await;
//
//         Ok(())
//     }
//
//     fn activate_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//         let subscriber = tracing_subscriber::fmt()
//             .with_env_filter("slack_morphism=debug")
//             .finish();
//         tracing::subscriber::set_global_default(subscriber)?;
//         Ok(())
//     }
//
//     fn get_app_token() -> SlackApiToken {
//         let app_token_value: SlackApiTokenValue = CONFIG.app_token.clone();
//         let app_token: SlackApiToken = SlackApiToken::new(app_token_value);
//         app_token
//     }
//
//     fn build_listener(
//         listener_environment: Arc<
//             SlackClientEventsListenerEnvironment<
//                 SlackClientHyperConnector<HttpsConnector<HttpConnector>>,
//             >,
//         >,
//     ) -> SlackClientSocketModeListener<SlackClientHyperConnector<HttpsConnector<HttpConnector>>>
//     {
//         SlackClientSocketModeListener::new(
//             &SlackClientSocketModeConfig::new(),
//             listener_environment,
//             Self::build_callbacks(),
//         )
//     }
//
//     fn build_listener_environment(
//         self,
//         client: Arc<SlackClient<SlackClientHyperConnector<HttpsConnector<HttpConnector>>>>,
//     ) -> Arc<
//         SlackClientEventsListenerEnvironment<
//             SlackClientHyperConnector<HttpsConnector<HttpConnector>>,
//         >,
//     > {
//         Arc::new(
//             SlackClientEventsListenerEnvironment::new(client)
//                 .with_error_handler(on_error::on_error)
//                 .with_user_state(self)
//                 .with_user_state(DefaultActionHandler::new()),
//         )
//     }
//
//     fn build_callbacks() -> SlackSocketModeListenerCallbacks<SlackClientHyperHttpsConnector> {
//         SlackSocketModeListenerCallbacks::new()
//             .with_command_events(on_command::on_command_event)
//             .with_interaction_events(on_interaction::on_interaction_event)
//             .with_push_events(on_push::on_push_event)
//     }
// }
