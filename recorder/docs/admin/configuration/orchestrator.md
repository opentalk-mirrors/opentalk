# Orchestrator

The {{ product_name }} Orchestrator coordinates recording jobs across multiple
Recorder instances. Connecting to an Orchestrator is optional: if the
`orchestrator` section is omitted from the configuration file, the Recorder
runs without Orchestrator integration.

## Configuration

The section in the [configuration file](README.md) is called `orchestrator`.

| Field     | Type              | Required | Default value | Description                                                                                                                                   |
| --------- | ----------------- | -------- | ------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| `url`     | `string`          | yes      | -             | The base URL of the Orchestrator. Must use `http`/`https`, must not contain a query string, and if it has a path that path must end with `/`. |
| `api_key` | `string`/`ApiKey` | yes      | -             | The API key for the Orchestrator's service API                                                                                                |

### API Key

The API Key for the Orchestrator can be configured as string (`"<key_id>:<key_secret>"`) or as key/value pair (`{ id = "<key_id>", secret = "<key_secret>" }`)

### Example

```toml
[orchestrator]
url = "http://127.0.0.1:11222"
api_key = { "id" = "orchestrator", "secret" = "examplesecret" }
```
