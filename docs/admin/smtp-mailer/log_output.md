# Log Output

The log output of the {{ product_name }} SMTP Mailer can be configured, allowing administrators to control the
verbosity and granularity of log messages.

## Log level

The verbosity of the log output of the {{ product_name }} SMTP Mailer is configured by setting a log level. The available log levels
are: "error", "warn", "info", "debug" and "trace". Next to those its also possible to set the log level to "off" which
completely turns of the logging. By default the log level of the {{ product_name }} SMTP Mailer is set to "warn". For more information
on these log levels see the [documentation of the log crate](https://docs.rs/log/latest/log/enum.Level.html).

## Setting the log level

The log level of the {{ product_name }} SMTP Mailer can be configured with the `SMTP_MAILER_LOG` or `RUST_LOG` environment variable, where `SMTP_MAILER_LOG` takes precedence. So for example, if you
want to set the global log level to "debug" you could start the {{ product_name }} SMTP Mailer with this command:

```sh
RUST_LOG=debug opentalk-smtp-mailer
```

Next to the global log level you can set the log level of specific modules. This is done by providing a key-value pair, where
the key is the name of the module and the key is the log level you want to set for this module. These pairs can be chained
together in a comma seperated list, if you want to set the log level for multiple modules. So for example, if you want to start
the {{ product_name }} SMTP Mailer with the global log level set to "info" and the log level of the `opentalk_smtp_mailer::settings`
module set to"warn" you could run the following command:

```sh
RUST_LOG=debug,opentalk_smtp_mailer::settings=warn opentalk-smtp-mailer
```

### Special cases

Some crates we depend on produce very verbose output at the "debug" level, which
makes the log output hard to navigate. The built-in default directives therefore
cap the following crates at "warn":

- [html5ever](https://crates.io/crates/html5ever)
- [selectors](https://crates.io/crates/selectors)

The log filter is taken from a single source (see
[Filter directive precedence](#filter-directive-precedence)); the sources are
not merged. When you set `RUST_LOG`, it fully replaces the built-in defaults, so
these caps no longer apply automatically. Include them explicitly if you want to
keep them, for example:

```sh
RUST_LOG=debug,html5ever=warn,selectors=warn opentalk-smtp-mailer
```

## Filter directive precedence

The log filter directives are resolved from the first source that is set, in
this order:

1. The `SMTP_MAILER_LOG` environment variable.
2. The `RUST_LOG` environment variable.
3. The `default_directives` option in the `[tracing]` configuration section.
4. The built-in defaults (`warn`, with `html5ever` and `selectors` capped at
   `warn`).

Only one source is used; they are not merged.

## OpenTelemetry tracing

The {{ product_name }} SMTP Mailer can export spans to an
[OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. Configure it in the
optional `[tracing]` section of the configuration file:

| Field | Type | Required | Default value | Description |
| ----- | ---- | -------- | ------------- | ----------- |
| `otlp_tracing_endpoint` | `string` | no | - | OTLP gRPC endpoint; when unset, trace export is disabled |
| `service_name` | `string` | no | `smtp-mailer` | OpenTelemetry service name |
| `service_namespace` | `string` | no | `opentalk` | OpenTelemetry service namespace |
| `service_instance_id` | `string` | no | random UUID | OpenTelemetry service instance ID |
| `default_directives` | `array` | no | `["warn", "html5ever=warn", "selectors=warn"]` | Log and span filter used when neither `SMTP_MAILER_LOG` nor `RUST_LOG` is set |

```toml
[tracing]
otlp_tracing_endpoint = "http://localhost:4317"
default_directives = ["warn", "html5ever=warn", "selectors=warn"]
```
