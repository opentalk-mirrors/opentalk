# Monitoring

The {{ product_name }} Obelisk provides a simple Http-Server for monitoring purpose. This HTTP service is only provided when configured.

The [health command](health.md) can be used to determine the readiness state.

## Configuration

The section in the [configuration file](README.md) is called `monitoring`.
If this section is kept empty, it means that the `monitoring` is disabled.

| Field  | Type     | Required | Default value | Description                            |
| ------ | -------- | -------- | ------------- | -------------------------------------- |
| `port` | `int`    | no       | 11411         | The port for the monitoring server.    |
| `addr` | `string` | no       | 0.0.0.0       | The address for the monitoring server. |

### Example

```toml
[monitoring]
port = 8001
addr = "0.0.0.0"
```
