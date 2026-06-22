# Auth

The Transcription service authenticates incoming requests on
its internal endpoints with API keys.

## Configuration

The API keys are configured in the [`http`](http.md) section of the
[configuration file](README.md) through the `api_keys` field.

The transcription service can have multiple API keys configured. An API key can
be configured as a string (`"<key_id>:<key_secret>"`) or as a key/value pair
(`{ id = "<key_id>", secret = "<key_secret>" }`).

| Field    | Type     | Required | Description                       |
| -------- | -------- | -------- | --------------------------------- |
| `id`     | `string` | yes      | The identifier of the API key.    |
| `secret` | `string` | yes      | The secret value of the API key.  |

### Example

```toml
[http]
api_keys = [{ id = "transcription", secret = "secret" }]
```
