# Termibot v3

[![Tests](https://github.com/Mattsi-Jansky/termibot/actions/workflows/main.yml/badge.svg)](https://github.com/Mattsi-Jansky/termibot/actions/workflows/main.yml)
[![Security audit](https://github.com/Mattsi-Jansky/termibot/actions/workflows/audit.yml/badge.svg)](https://github.com/Mattsi-Jansky/termibot/actions/workflows/audit.yml)

A bot for a private Slack.

## Dependencies

* [Rust](https://www.rust-lang.org/tools/install)
* `sudo apt install libssl-dev libsqlite3-dev`

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

## Stretch Goals

* Create some sort of DSL to make creating blocks API messages easier
  * A simple solution: Write it in JSON, use `format!` to inject values and `serde` to deserialise it?
* Improve the Songlink plugin
  * Give output similar to Songlink app itself?
* Generate previews for Mastodon
 * Currently Slack only shows first image of toots that have multiple images, would be great to be able to add the other images
 * Would require matching against a long list of Mastoon instance URLs, can never be comprehensive.
