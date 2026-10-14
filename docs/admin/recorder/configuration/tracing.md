# Tracing

The {{ product_name }} Recorder can export spans to an
[OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. When tracing is not
configured, log output continues to use the existing `RECORDER_LOG` and
`RUST_LOG` directives.

## Configuration

The section in the [configuration file](README.md) is called `tracing`.

| Field | Type | Required | Default value | Description |
| ----- | ---- | -------- | ------------- | ----------- |
| `otlp_tracing_endpoint` | `string` | no | - | OTLP gRPC endpoint |
| `service_name` | `string` | no | `recorder` | OpenTelemetry service name |
| `service_namespace` | `string` | no | `opentalk` | OpenTelemetry service namespace |
| `service_instance_id` | `string` | no | random UUID | OpenTelemetry service instance ID |
| `default_directives` | `array` | no | `["warn", "opentalk_recorder=info"]` | Log and span filter used when neither `RECORDER_LOG` nor `RUST_LOG` is set |

The log filter directives are resolved with the following precedence:
`RECORDER_LOG` > `RUST_LOG` > `default_directives` > the built-in defaults. Only
the first source that is set is used; the sources are not merged.

```toml
[tracing]
otlp_tracing_endpoint = "http://localhost:4317"
service_name = "recorder"
service_namespace = "opentalk"
```

## Trace context propagation

The Recorder uses the [W3C Trace Context](https://www.w3.org/TR/trace-context-1)
`traceparent` and `tracestate` HTTP headers. When OTLP tracing export is
enabled, valid context received on an HTTP request becomes the parent of the
Recorder request span, and internal HTTP requests to other services carry the
active context. When OTLP tracing is disabled (the default), these headers are
neither parsed nor emitted.

Configure all {{ product_name }} services to export to the same collector or
tracing backend to display their spans as one trace in tools such as Jaeger.
Propagation currently covers HTTP traffic between these services. Services
without OTLP tracing export terminate the visible trace, even when they receive
a `traceparent` header.

!!! warning "Treat incoming trace headers as untrusted"

    When tracing is enabled, `traceparent` and `tracestate` are accepted from any
    client. To avoid spoofed or malformed trace context, strip or overwrite these
    headers at a trusted reverse proxy for requests from untrusted sources, as
    recommended by the [OpenTelemetry security best practices](https://opentelemetry.io/docs/concepts/context-propagation/#security-best-practices).
