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

### `CONTROLLER_LOG` and `RUST_LOG` environment variables

The log level of the controller can also be configured with the `CONTROLLER_LOG` or `RUST_LOG` environment variable, where
`CONTROLLER_LOG` takes precedence. Both follow the same pattern as the `default_directives` field. The log filter is taken
from a single source; the sources are not merged. The precedence is: `CONTROLLER_LOG` > `RUST_LOG` > `default_directives`
from the `config.toml` > built-in defaults. When you set `CONTROLLER_LOG` or `RUST_LOG`, it fully replaces the
`default_directives`.

For example, the following command sets the log level for the `opentalk` component, replacing any directives configured
in the `config.toml`:

```sh
CONTROLLER_LOG=opentalk=DEBUG cargo run
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
