---
title: Exporting the OpenAPI specification
---

# OpenAPI Specification

The OpenAPI specification can be exported from the Command-Line interface using a subcommand.

## `opentalk-roomserver openapi` subcommand

This subcommand is the top-level entrypoint to exporting the OpenAPI specification of the web api.

Help output looks like this:

<!-- begin:fromfile:cli-usage/openapi-help.md -->

```text
OpenAPI related commands

Usage: opentalk-roomserver openapi <COMMAND>

Commands:
  dump  Store the OpenAPI schema
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- end:fromfile:cli-usage/openapi-help.md -->

## `opentalk-roomserver openapi dump` subcommand

This subcommand allows to dump the OpenAPI specification to either `stdout` or a file.

Help output looks like this:

<!-- begin:fromfile:cli-usage/openapi-dump-help.md -->

```text
Store the OpenAPI schema

Usage: opentalk-roomserver openapi dump [OPTIONS] [TARGET]

Arguments:
  [TARGET]
          The output target

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

<!-- end:fromfile:cli-usage/openapi-dump-help.md -->
