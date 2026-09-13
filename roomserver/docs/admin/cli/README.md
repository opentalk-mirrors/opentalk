---
title: Command-Line Usage
---

# Command-Line Usage of `opentalk-roomserver`

When started without a subcommand, the `opentalk-roomserver` command loads the [configuration](../configuration.md) and starts as a service.

In addition, some subcommands are available for management tasks.

## Subcommands

These subcommands are available:

- [`health`](health.md#opentalk-roomserver-health-subcommand) for fetching the readiness state.
- [`openapi`](openapi.md#opentalk-roomserver-openapi-subcommand) for exporting the OpenAPI specification.
- `help` for showing the help output.

## Raw help output

Help output looks like this:

<!-- begin:fromfile:cli-usage/help.md -->

```text
OpenTalk RoomServer

Usage: opentalk-roomserver [OPTIONS] [COMMAND]

Commands:
  health   Return the readiness state
  openapi  OpenAPI related commands
  modules  Command show available modules
  help     Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>
          Path of the configuration file.

          If present, exactly this config file will be used.

          If absent, `roomserver` looks for a config file in these locations and uses the first one that is found:

          - `roomserver.toml` in the current directory - `<XDG_CONFIG_HOME>/opentalk/roomserver.toml` (where `XDG_CONFIG_HOME` is usually `~/.config`) - `/etc/opentalk/roomserver.toml`

  -V, --version
          Print version information

  -l, --license
          Print license information

  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:cli-usage/help.md -->
