---
title: Tracing
---

# Tracing

The {{ product_name }} Obelisk can export spans to an
[OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. When no endpoint is
configured, the service only writes formatted log output and applies the
`OBELISK_LOG` and `RUST_LOG` directives.

## Configuration

The section in the [configuration file](configuration.md) is called `tracing`.
The whole section is optional.

| Field | Type | Required | Default value | Description |
| ----- | ---- | -------- | ------------- | ----------- |
| `otlp_tracing_endpoint` | `string` | no | - | OTLP gRPC endpoint; when unset, trace export is disabled |
| `service_name` | `string` | no | `obelisk` | OpenTelemetry service name |
| `service_namespace` | `string` | no | `opentalk` | OpenTelemetry service namespace |
| `service_instance_id` | `string` | no | random UUID | OpenTelemetry service instance ID |
| `default_directives` | `array` | no | `["warn", "opentalk_obelisk=info"]` | Log and span filter used when neither `OBELISK_LOG` nor `RUST_LOG` is set |

The log filter directives are resolved with the following precedence:
`OBELISK_LOG` > `RUST_LOG` > `default_directives` > the built-in defaults. Only
the first source that is set is used; the sources are not merged.

```toml
[tracing]
otlp_tracing_endpoint = "http://localhost:4317"
service_name = "obelisk"
service_namespace = "opentalk"
default_directives = ["warn", "opentalk_obelisk=info"]
```

## Trace context propagation

The Obelisk uses the [W3C Trace Context](https://www.w3.org/TR/trace-context-1)
`traceparent` and `tracestate` HTTP headers. When OTLP tracing export is
enabled, internal HTTP requests to other services carry the active context, for
example when a dial-in participant joins. The Obelisk does not accept trace
context from incoming requests. When OTLP tracing is disabled (the default),
these headers are not emitted.

Configure all {{ product_name }} services to export to the same collector or
tracing backend to display their spans as one trace in tools such as Jaeger.
Propagation currently covers HTTP traffic between these services. Services
without OTLP tracing export terminate the visible trace, even when they receive
a `traceparent` header.
