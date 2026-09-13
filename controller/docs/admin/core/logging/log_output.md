# Log Output

The log output from each component of the {{ product_name }} Controller can be configured, allowing administrators to control the
verbosity and granularity of log messages.

## Configuration

The section in the [configuration file](../configuration.md) is called `logging`. The examples here will only cover
the log output of the controller. The rest of the fields of the `logging` section are related to the `tracing` configuration.
See [tracing](./tracing.md) for more information.

| Field                | Type       | Required | Default value                                                                         | Description                                                              |
| -------------------- | ---------- | -------- | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| `default_directives` | `string[]` | no       | `["ERROR","opentalk=INFO","pinky_swear=OFF","rustls=WARN","mio=ERROR","lapin=WARN",]` | The global log level as well as a list of components and their log level |

One of the values in the list of the `default_directives` can be the global log level, being either `OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG` or `TRACE`.
The global log level affects all components that don't have a specific log level configured. The default global log level is `ERROR`.

The other values in the `default_directives` list should be key-value pairs with the key being the component and the value being the log level.

To change the default values (`["ERROR","opentalk=INFO","pinky_swear=OFF","rustls=WARN","mio=ERROR","lapin=WARN",]`) of
the `default_directives`, each component must be explicitly overwritten in the `config.toml`, otherwise the default values
persist alongside the other configured directives.

### `RUST_LOG` environment variable

The log level of the controller can also be configured with the `RUST_LOG` environment variable. The variable follows the
same pattern as the `default_directives` field. Any manually set values for the `default_directives` will still be applied
alongside the `RUST_LOG` values. However, in case of a conflict, the values from the `RUST_LOG` are prioritized higher
than the `default_directives` from the `config.toml`.

For example, the following command would overwrite any log level values for the `opentalk` component if they were set
in the `config.toml`:

```sh
RUST_LOG=opentalk=DEBUG cargo run
```

## Examples

### Set the global log level to `WARN`

```toml
[logging]
default_directives = [
  "WARN",
]
```

### Default Setup

```toml
[logging]
default_directives = [
  "ERROR",
  "opentalk=INFO",
  "pinky_swear=OFF",
  "rustls=WARN",
  "mio=ERROR",
  "lapin=WARN",
]
```
