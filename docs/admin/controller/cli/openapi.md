---
sidebar_position: 107
title: Exporting the OpenAPI specification
---

# OpenAPI Specification

The OpenAPI specification can be exported from the Command-Line interface using a subcommand.

## `opentalk-controller openapi` subcommand

This subcommand is the top-level entrypoint to exporting the OpenAPI specification of the web api.

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-openapi-help.md -->

```text
Get information on the OpenAPI specification

Usage: opentalk-controller openapi <COMMAND>

Commands:
  dump  Dump the OpenAPI specification
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-openapi-help.md -->

## `opentalk-controller openapi dump` subcommand

This subcommand allows to dump the OpenAPI specification to either `stdout` or a file.

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-openapi-dump-help.md -->

```text
Dump the OpenAPI specification

Usage: opentalk-controller openapi dump [OPTIONS] [TARGET]

Arguments:
  [TARGET]
          The output target (either a filename, or `-` for stdout)

          [default: -]

Options:
      --format <FORMAT>
          The export format

          Possible values:
          - yaml: YAML output format
          - json: JSON output format

          [default: yaml]

  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:cli-usage/opentalk-controller-openapi-dump-help.md -->
