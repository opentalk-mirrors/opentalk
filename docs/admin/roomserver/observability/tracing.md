# Tracing

The {{ product_name }} RoomServer is able to provide tracing information. If configured, these are exported to an [OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. Such an endpoint can be provided by a tracing platform such as [Jaeger](https://www.jaegertracing.io/).

## Configuration

The configuration values for the tracing capabilities are in the `tracing` section of the [configuration file](../configuration.md).

| Field                   | Type     | Required | Default value | Description                               |
| ----------------------- | -------- | -------- | ------------- | ----------------------------------------- |
| `otlp_tracing_endpoint` | `string` | no       | -             | OTLP tracing endpoint to export traces to |

## Trace context propagation

The RoomServer uses the
[W3C Trace Context](https://www.w3.org/TR/trace-context-1) `traceparent` and
`tracestate` HTTP headers. When OTLP tracing export is enabled, valid context
received on an HTTP request becomes the parent of the RoomServer request span,
and internal HTTP requests to other services carry the active context. When OTLP
tracing is disabled (the default), these headers are neither parsed nor emitted.

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
