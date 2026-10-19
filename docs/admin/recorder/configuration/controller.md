# Controller

## Configuration

The section in the [configuration file](README.md) is called `controller`.

| Field               | Type              | Required | Default value | Description                                                                                                                               |
| ------------------- | ----------------- | -------- | ------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `url`               | `string`          | yes      | -             | The URL of the Controller                                                                                                                 |
| `api_key`           | `string`/`ApiKey` | yes      | -             | The API key for the Controllers service API                                                                                               |
| `upload_chunk_size` | `int`             | no       | 5242880       | The size for each chunk on upload. Expected the value to lie between 5 MiB and 5 GiB. The value is represented in Bytes. (Default: 5 MiB) |

### API Key

The API Key for the Controller can be configured as string (`"<key_id>:<key_secret>"`) or as key/value pair (`{ id = "<key_id>", secret = "<key_secret>" }`)

### Example

```toml
[controller]
url = "http://localhost:8000"
api_key = { "id" = "controller", "secret" = "examplesecret" }
upload_chunk_size = 10485760 # 10 MiB
```
