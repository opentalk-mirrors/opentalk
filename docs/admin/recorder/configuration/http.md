# HTTP Server

The OpenTalk Recorder provides its functionality to clients through a built-in HTTP server.

## Configuration

The section in the [configuration file](README.md) is called `http`.

| Field      | Type                       | Required | Default value | Description                                           |
| ---------- | -------------------------- | -------- | ------------- | ----------------------------------------------------- |
| `port`     | `int`                      | no       | 11511         | The port for the http server.                         |
| `addr`     | `string`                   | no       | 0.0.0.0       | The address used for the http server.                 |
| `api_keys` | array of `string`/`ApiKey` | yes      | -             | The API keys accepted for internal service endpoints. |

### API Keys

The recorder can have multiple API keys configured for authenticating internal
service endpoints. Each entry can be configured as string
(`"<key_id>:<key_secret>"`) or as key/value pair (`{ id = "<key_id>", secret = "<key_secret>" }`).

### Example

```toml
[http]
port = 8080
addr = "0.0.0.0"
api_keys = [
    { id = "roomserver", secret = "secret1" },
    "controller:very_secret",
]
```
