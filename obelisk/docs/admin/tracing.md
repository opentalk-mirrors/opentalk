---
title: Tracing
---

# Tracing

The {{ product_name }} Obelisk can export spans to an
[OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. When no endpoint is
configured, the service only writes formatted log output and applies the
`RUST_LOG` directives.

## Configuration

The section in the [configuration file](configuration.md) is called `tracing`.
The whole section is optional.

| Field | Type | Required | Default value | Description |
| ----- | ---- | -------- | ------------- | ----------- |
| `otlp_tracing_endpoint` | `string` | no | - | OTLP gRPC endpoint; when unset, trace export is disabled |
| `service_name` | `string` | no | `obelisk` | OpenTelemetry service name |
| `service_namespace` | `string` | no | `opentalk` | OpenTelemetry service namespace |
| `service_instance_id` | `string` | no | random UUID | OpenTelemetry service instance ID |
| `default_directives` | `array` | no | `["warn", "opentalk_obelisk=info"]` | Log and span filter used when `RUST_LOG` is unset |

The log filter directives are resolved with the following precedence:
`RUST_LOG` > `default_directives` > the built-in defaults. Only the first source
that is set is used; the sources are not merged.

```toml
[tracing]
otlp_tracing_endpoint = "http://localhost:4317"
service_name = "obelisk"
service_namespace = "opentalk"
default_directives = ["warn", "opentalk_obelisk=info"]
```
