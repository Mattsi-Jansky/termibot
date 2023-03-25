# Termibot v3

We're going to Rust town, baby!

## How to use

* If you haven't already, [create a Slack app](https://api.slack.com/authentication/basics)
  * Enable socket mode
  * Enable events API and give it access to all the bot messing read/write events, reactions and threads
  * Enable OAuth2 and give it access to all the same bot events
  * Only needs "bot" events, not "user" events
* Get the bot's "app token" from the "basic info" tab of the app's page, toward the bottom
* Get the bot's "bot token" from the OAuth tab of the app's page
* Copy `config/template.toml` into `config/config.toml`
  * Note this directory is gitignored to prevent inadvertently pushing secure tokens
* Add the app token and bot token to `config/config.toml`
* `cargo run`

## Goals

* Extensible architecture
* Karma tracker
* Convert non-Songlink links to Songlink
  * Give output similar to Songlink app itself?
* Each time a new emoji is added, post the "emoji changelist" automatically
