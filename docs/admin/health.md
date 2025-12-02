# Health command

Fetching the current readiness state from the [monitoring endpoint](./monitoring.md ) via cli interface using a subcommand.

The endpoint id determined:

- If the endpoint argument is present, it will be used regardless the configuration of the obelisk.
- If not the endpoint will be loaded from the [monitoring configuration](./monitoring.md).
- Without argument and existing configuration the command fails.

## `opentalk-controller health` subcommand

Help output looks like this:

```text
Return the readiness state

Usage: opentalk-obelisk health [ENDPOINT]

Arguments:
  [ENDPOINT]  The monitoring endpoint can be provided optionally

Options:
  -h, --help  Print help
```
