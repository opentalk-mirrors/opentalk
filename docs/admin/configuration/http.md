# HTTP Server

The Transcription service provides its functionality to
clients through a built-in HTTP server.

## Configuration

The section in the [configuration file](README.md) is called `http`.

| Field      | Type     | Required | Default value | Description                            |
| ---------- | -------- | -------- | ------------- | -------------------------------------- |
| `addr`     | `string` | no       | `0.0.0.0`     | The address used for the HTTP server.  |
| `port`     | `int`    | no       | `11411`       | The port for the HTTP server.          |
| `api_keys` | `list`   | yes      | -             | The API keys for the service endpoints. See [Auth](auth.md). |

### Example

```toml
[http]
addr = "0.0.0.0"
port = 11411
api_keys = [{ id = "transcription", secret = "secret" }]
```
