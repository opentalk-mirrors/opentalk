# Tracing

The {{ product_name }} RoomServer is able to provide tracing information. If configured, these are exported to an [OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. Such an endpoint can be provided by a tracing platform such as [Jaeger](https://www.jaegertracing.io/).

## Configuration

The configuration values for the tracing capabilities are in the `tracing` section of the [configuration file](../configuration.md).

| Field                   | Type     | Required | Default value | Description                               |
| ----------------------- | -------- | -------- | ------------- | ----------------------------------------- |
| `otlp_tracing_endpoint` | `string` | no       | -             | OTLP tracing endpoint to export traces to |
