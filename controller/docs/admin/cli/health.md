---
sidebar_position: 107
title: Fetch the current readiness state
---

# Health command

Fetching the current readiness state from the [monitoring endpoint](../core/monitoring.md) via cli interface using a subcommand.

The endpoint id determined:

- If the endpoint argument is present, it will be used regardless the configuration of the controller.
- If not the endpoint will be loaded from the [monitoring configuration](../core/monitoring.md).
- Without argument and existing configuration the command fails.

## `opentalk-controller health` subcommand

This subcommand is the top-level entrypoint to exporting the OpenAPI specification of the web api.

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-health-help.md -->

```text
Return the readiness state

Usage: opentalk-controller health [ENDPOINT]

Arguments:
  [ENDPOINT]  The monitoring endpoint can be provided optionally

Options:
  -h, --help  Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-health-help.md -->
