# Tracing

The {{ product_name }} Transcription service can export spans to an
[OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. When no endpoint is
configured, the service only writes formatted log output and applies the
`TRANSCRIPTION_LOG` and `RUST_LOG` directives.

## Configuration

The section in the [configuration file](README.md) is called `tracing`. The
whole section is optional.

| Field | Type | Required | Default value | Description |
| ----- | ---- | -------- | ------------- | ----------- |
| `otlp_tracing_endpoint` | `string` | no | - | OTLP gRPC endpoint; when unset, trace export is disabled |
| `service_name` | `string` | no | `transcription` | OpenTelemetry service name |
| `service_namespace` | `string` | no | `opentalk` | OpenTelemetry service namespace |
| `service_instance_id` | `string` | no | random UUID | OpenTelemetry service instance ID |
| `default_directives` | `array` | no | `["warn", "opentalk_transcription=info"]` | Log and span filter used when neither `TRANSCRIPTION_LOG` nor `RUST_LOG` is set |

The log filter directives are resolved with the following precedence:
`TRANSCRIPTION_LOG` > `RUST_LOG` > `default_directives` > the built-in defaults.
Only the first source that is set is used; the sources are not merged.

```toml
[tracing]
otlp_tracing_endpoint = "http://localhost:4317"
service_name = "transcription"
service_namespace = "opentalk"
default_directives = ["warn", "opentalk_transcription=info"]
```
