# Controller

The {{ product_name }} Controller serves the storage API that is used by the RoomServer to save assets and module resources.
When no Controller is configured, the RoomServer assets will be stored in local memory and can't be accessed after the
RoomServer has stopped.

## Configuration

The section in the [configuration file](configuration.md) is called `controller`.

| Field     | Type              | Required | Default value | Description                                 |
| --------- | ----------------- | -------- | ------------- | ------------------------------------------- |
| `url`     | `string`          | yes      | -             | The URL of the Controller                   |
| `api_key` | `string`/`ApiKey` | yes      | -             | The API key for the Controllers service API |

### API Key

The API Key for the Controller can be configured as string (`<key_id>:<key_secret>`) or as key/value pair (`{id = "<key_id>", secret = "<key_secret>"}`)
