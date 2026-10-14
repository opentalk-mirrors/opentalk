# Monitoring

The Transcription service provides a simple HTTP server for
monitoring purposes. It exposes a readiness probe used by the `health`
subcommand and is the source of the service's readiness state.

## Configuration

The section in the [configuration file](README.md) is called `monitoring`. If
this section is absent, monitoring is disabled.

| Field  | Type     | Required | Default value | Description                                 |
| ------ | -------- | -------- | ------------- | ------------------------------------------- |
| `addr` | `string` | no       | `0.0.0.0`     | The address used for the monitoring server. |
| `port` | `int`    | no       | `11411`       | The port for the monitoring server.         |

### Example

```toml
[monitoring]
addr = "0.0.0.0"
port = 11411
```
