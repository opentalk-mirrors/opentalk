---
title: Command-Line usage
---

# Command-Line Usage of `opentalk-controller`

When started without a subcommand, the `opentalk-controller` command loads the
[configuration](../core/configuration.md) and starts as a service, serving incoming
connections using the included [HTTP server](../core/http_server.md).

In addition, some subcommands are available for management tasks.

## Subcommands

These subcommands are available:

- [`migrate-db`](../core/database.md#opentalk-controller-migrate-db-subcommand) for explicit migration of database without starting the controller service
- [`tenants`](../advanced/tenants.md#opentalk-controller-tenants-subcommand) for managing tenants.
- [`tariffs`](../advanced/tariffs.md#opentalk-controller-tariffs-subcommand) for managing tariffs.
- [`jobs`](jobs.md#opentalk-controller-jobs-subcommand) for configuring and running maintenance jobs.
- [`modules`](../advanced/modules.md#opentalk-controller-modules-subcommand) for managing modules.
- [`health`](health.md#opentalk-controller-health-subcommand) for fetching the current readiness state.
- [`openapi`](openapi.md#opentalk-controller-openapi-subcommand) for exporting the OpenAPI specification.
- `help` for showing the help output.

## Raw help output

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-help.md -->

```text
Usage: opentalk-controller [OPTIONS] [COMMAND]

Commands:
  migrate-db  Migrate the db. This is done automatically during start of the controller, but can be done without starting the controller using this command
  tenants     Manage existing tenants
  tariffs     Manage tariffs
  jobs        Manage and execute maintenance jobs
  modules     Manage modules
  openapi     Get information on the OpenAPI specification
  health      Return the readiness state
  reload      Triggers a reload of reloadable configuration options for already running opentalk-controller processes
  help        Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>
          Path of the configuration file.

          If present, exactly this config file will be used.

          If absent, `controller` looks for a config file in these locations and uses the first one that is found:

          - `config.toml` in the current directory (deprecated, for backwards compatiblity only)
          - `controller.toml` in the current directory
          - `<XDG_CONFIG_HOME>/opentalk/controller.toml` (where `XDG_CONFIG_HOME` is usually `~/.config`)
          - `/etc/opentalk/controller.toml`

      --reload
          Triggers a reload of reloadable configuration options (deprecated, use the `reload` subcommand instead)

  -V, --version
          Print version information

  -l, --license
          Print license information

  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:cli-usage/opentalk-controller-help.md -->
