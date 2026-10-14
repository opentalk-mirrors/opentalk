# Controller

The Transcription service connects to the
Controller to join rooms and to send transcript segments back.

## Configuration

The section in the [configuration file](README.md) is called `controller`.

| Field     | Type     | Required | Description                                       |
| --------- | -------- | -------- | ------------------------------------------------- |
| `url`     | `string` | yes      | The base URL of the controller.                   |
| `api_key` | `table`  | yes      | The API key used to authenticate the controller.  |

The `api_key` is a key/value pair of `id` and `secret`.

### Example

```toml
[controller]
url = "http://localhost:8000"
api_key = { id = "controller", secret = "secret" }
```
