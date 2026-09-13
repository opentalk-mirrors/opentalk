# Whiteboard (Spacedeck)

The {{ product_name }} Controller uses a [Spacedeck](https://github.com/spacedeck) server to provide a collaborative whiteboard for the `whiteboard` module.

When no Spacedeck is configured, the `whiteboard` module is disabled.

## Deploy Spacedeck

A `Spacedeck` instance is required before any configuration can be done. You can find a modified Spacedeck image in our
[Open CoDE Spacedeck repository](https://gitlab.opencode.de/opentalk/spacedeck).

You can clone the repository and build the Docker image yourself or download the image directly from the [Container Registry](https://gitlab.opencode.de/opentalk/spacedeck/container_registry)

When you start the Docker container, make sure to configure the `SD_API_TOKEN` environment variable. The value of that
variable will be the API token that the Controller uses for requests to the Spacedeck API.

## Configuration

The section in the [configuration file](../../core/configuration.md) is called `spacedeck`.

| Field     | Type     | Required | Default value | Description                                                                                                                                                          |
| --------- | -------- | -------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `url`     | `string` | yes      | -             | The URI address where the Spacedeck server can be reached                                                                                                            |
| `api_key` | `string` | yes      | -             | The [API key](https://gitlab.opencode.de/opentalk/spacedeck/-/blob/main/README.md?ref_type=heads#configure-api-token) that is used for requests to the Spacedeck API |

### Examples

#### Default Setup

```toml
[spacedeck]
url = "http://localhost:9666"
api_key = "secret"
```
