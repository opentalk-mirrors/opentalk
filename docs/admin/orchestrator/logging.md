# Logging

The {{ product_name }} Orchestrator writes its log output to stdout. The verbosity can be controlled per module using
logging directives.

## Configuration

The section in the [configuration file](README.md) is called `logging`. The section is optional.

| Field                   | Type       | Required | Default value                            | Description                       |
| ----------------------- | ---------- | -------- | ---------------------------------------- | --------------------------------- |
| `directives`            | `string[]` | no       | `["warn", "opentalk_orchestrator=info"]` | The logging directives to apply   |
| `otlp_tracing_endpoint` | `string`   | no       | -                                        | OTLP gRPC endpoint                |
| `service_name`          | `string`   | no       | `orchestrator`                           | OpenTelemetry service name        |
| `service_namespace`     | `string`   | no       | `opentalk`                               | OpenTelemetry service namespace   |
| `service_instance_id`   | `string`   | no       | random UUID                              | OpenTelemetry service instance ID |

### Directives

The directives use the same syntax as the `RUST_LOG` environment variable. Each entry is either a global log level
(`error`, `warn`, `info`, `debug`, `trace` or `off`) or a `<module>=<level>` pair that sets the log level of a
specific module.

### Precedence

The directives are taken from the first of these sources that is set and not empty:

1. The `ORCHESTRATOR_LOG` environment variable
2. The `RUST_LOG` environment variable
3. The `directives` field of the `logging` section
4. The default value `warn,opentalk_orchestrator=info`

```sh
ORCHESTRATOR_LOG=info,opentalk_orchestrator=debug opentalk-orchestrator
```

### Example

```toml
[logging]
directives = ["info", "opentalk_orchestrator=debug"]
otlp_tracing_endpoint = "http://localhost:4317"
service_name = "orchestrator"
service_namespace = "opentalk"
```
