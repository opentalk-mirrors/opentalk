# Orchestrator

The Transcription service can optionally register with an
orchestrator, which distributes transcription jobs across multiple instances. If
this section is absent, the orchestrator integration is disabled.

## Configuration

The section in the [configuration file](README.md) is called `orchestrator`.

| Field     | Type     | Required | Description                                         |
| --------- | -------- | -------- | --------------------------------------------------- |
| `url`     | `string` | yes      | The URL of the orchestrator.                        |
| `api_key` | `table`  | yes      | The API key used to authenticate the orchestrator.  |

The `api_key` is a key/value pair of `id` and `secret`.

### Example

```toml
[orchestrator]
url = "http://127.0.0.1:11222"
api_key = { id = "orchestrator", secret = "secret" }
```
