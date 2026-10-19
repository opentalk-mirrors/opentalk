# Frontend

For some functionalities the controller may require information on the primary
frontend that is used. The link in invite emails uses the `base_url` for example.

## Configuration

The section in the [configuration file](configuration.md) is called `frontend`.

| Field        | Type     | Required | Default value | Description                   |
| ------------ | -------- | -------- | ------------- | ----------------------------- |
| `base_url`   | `string` | yes      | -             | The base URL of the frontend  |

Besides the usage in e.g. generating links, the `base_url` field value is also
used in the [CORS header](http_server.md#cors-configuration) unless overridden.

### Example configuration

```toml
[frontend]
base_url = "https://example.com"
```
