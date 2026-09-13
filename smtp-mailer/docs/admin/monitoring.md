# Monitoring

The {{ product_name }} SMTP Mailer provides a simple Http-Server for monitoring purpose.

## Configuration

The section in the [configuration file](README.md) is called `monitoring`.

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
