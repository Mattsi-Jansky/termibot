# Termibot v3

We're going to Rust town, baby!

# Dependencies

* [Rust](https://www.rust-lang.org/tools/install)
* `sudo apt install libssl-dev sqlite libsqlite3-dev libsqlite0-dev`

## How to use

* Ensure dependencies are installed
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

## Stretch Goals

* Create some sort of DSL to make creating blocks API messages easier
  * A simple solution: Write it in JSON, use `format!` to inject values and `serde` to deserialise it?
* Convert non-Songlink links to Songlink
  * Give output similar to Songlink app itself?
* Each time a new emoji is added, post the "emoji changelist" automatically
* Generate previews for Mastodon
 * Currently Slack only shows first image of toots that have multiple images, would be great to be able to add the other images
 * Would require matching against a long list of Mastoon instance URLs, can never be comprehensive.
