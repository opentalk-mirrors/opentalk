# Tracing

The {{ product_name }} Recorder can export spans to an
[OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. When tracing is not
configured, log output continues to use the existing `RUST_LOG` directives.

## Configuration

The section in the [configuration file](README.md) is called `tracing`.

| Field | Type | Required | Default value | Description |
| ----- | ---- | -------- | ------------- | ----------- |
| `otlp_tracing_endpoint` | `string` | no | - | OTLP gRPC endpoint |
| `service_name` | `string` | no | `recorder` | OpenTelemetry service name |
| `service_namespace` | `string` | no | `opentalk` | OpenTelemetry service namespace |
| `service_instance_id` | `string` | no | random UUID | OpenTelemetry service instance ID |
| `default_directives` | `array` | no | `["warn", "opentalk_recorder=info"]` | Log and span filter used when `RUST_LOG` is unset |

```toml
[tracing]
otlp_tracing_endpoint = "http://localhost:4317"
service_name = "recorder"
service_namespace = "opentalk"
```
