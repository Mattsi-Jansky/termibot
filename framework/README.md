# Slack bot client thingy

This is still quite experimental but this library represents a client for interacting with the Slack API.

## Tests

The integration tests require an actual Slack App setup in a Slack that has a `#bots` channel.

To run the integration tests first set a valid `bot_token` value in `config/config.toml` (gitignored, copy it from `config/template.toml`). Then run with `cargo test`.
