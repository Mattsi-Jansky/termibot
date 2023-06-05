# Slack bot client thingy

This is still quite experimental but this library represents a client for interacting with the Slack API.

## Tests

The tests use [RCVR](https://github.com/ChorusOne/rvcr/) to record and replay HTTP responses from the Slack API. They should run independently but if you want to make changes you'll need to record a new output from the Slack API (and check that the new response is valid!).

To run the tests against a real API first set a valid `bot_token` value in `config/config.toml` (gitignored, copy it from `config/template.toml`).

Run the tests with `cargo test`.
