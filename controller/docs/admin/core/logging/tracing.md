# Tracing

The {{ product_name }} controller is able to provide tracing information. If configured, these are exported to an [OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint.

## Configuration

The configuration values for the tracing capabilities are in the `logging` section of the [configuration file](../configuration.md).

| Field                   | Type     | Required | Default value                      | Description                                                       |
| ----------------------- | -------- | -------- | ---------------------------------- | ----------------------------------------------------------------- |
| `otlp_tracing_endpoint` | `string` | no       | -                                  | OTLP tracing endpoint to export traces to                         |
| `service_name`          | `string` | no       | `controller`                       | opentelemetry service name                                        |
| `service_namespace`     | `string` | no       | `opentalk`                         | opentelemetry service namespace                                   |
| `service_instance_id`   | `string` | no       | randomly generated UUID on startup | opentelemetry service instance id                                 |

### Examples

```toml
[logging]
otlp_tracing_endpoint = "http://localhost:4317"
service_name = "controller"
service_namespace = "opentalk"
service_instance_id = "627cc493-f310-47de-96bd-71410b7dec09"
```

This is not an exhaustive list of the configuration values in the logging section, just the ones related to tracing. For more information look into the [logging docs](./log_output.md).
