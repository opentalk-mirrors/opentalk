---
sidebar_position: 3
---

# Controller

## Configuration

The section in the [configuration file](configuration.md) is called `controller`.

| Field     | Type              | Required | Default value | Description                                 |
| --------- | ----------------- | -------- | ------------- | ------------------------------------------- |
| `url`     | `string`          | yes      | -             | The URL of the Controller                   |
| `api_key` | `string`/`ApiKey` | yes      | -             | The API key for the Controllers service API |

### API Key

The API Key for the Controller can be configured as string (<key_id>:<key_secret>) or as key/value pair ({id = "<key_id>", secret = "<key_secret>"})

### Example

```toml
[controller]
url = "http://localhost8000"
api_key = { "id" = "controller", "secret" = "examplesecret" }
```
