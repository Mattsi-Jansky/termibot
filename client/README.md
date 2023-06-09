# Slack bot client

A client for interacting with the Slack API.

## Tests

The tests use [RCVR](https://github.com/ChorusOne/rvcr/) to record and replay HTTP responses from the Slack API. They run with no dependencies but if you want to make changes you'll need to record a new output from the Slack API (and check that the new response is valid!). This is known as **record mode**.

To run the tests in record mode against the real Slack API first set a valid `bot_token` value in `config/config.toml` (gitignored, copy it from `config/template.toml`). Then, set `is_record_mode` to `true`. Remember to set it back to `false` before committing.

Run the tests with `cargo test`.
