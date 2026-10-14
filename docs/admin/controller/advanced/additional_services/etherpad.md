# Meeting Minutes (EtherPad)

The {{ product_name }} Controller uses an [Etherpad](https://etherpad.org/) server to provide a collaborative text editor for the `meeting_notes` module.

When no Etherpad is configured, the `meeting_notes` module is disabled.

## Deploy Etherpad

An Etherpad instance is required before any configuration can be done. You can find a pre-configured Etherpad in our
[Open CoDE Etherpad repository](https://gitlab.opencode.de/opentalk/etherpad).

You can clone the repository and build the Docker image yourself or download the image directly from the [Container Registry](https://gitlab.opencode.de/opentalk/etherpad/container_registry)

When you start the Docker container, make sure to configure the `EP_APIKEY` and `ADMIN_PASSWORD` environment variables as
described in the `Deployment` section of the [README.md](https://gitlab.opencode.de/opentalk/etherpad/-/blob/main/README.md?ref_type=heads#deployment).

Set the `TRUST_PROXY` environment variable if the Etherpad is running behind a reverse proxy.

## Configuration

The section in the [configuration file](../../core/configuration.md) is called `etherpad`.

| Field     | Type     | Required | Default value | Description                                                                                                                                            |
| --------- | -------- | -------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `url`     | `string` | yes      | -             | The URI address where the Etherpad server can be reached                                                                                               |
| `api_key` | `string` | yes      | -             | The [API key](https://gitlab.opencode.de/opentalk/etherpad/-/blob/main/README.md?ref_type=heads#api-key) that is used for requests to the Etherpad API |

### Examples

#### Default Setup

```toml
[etherpad]
url = "http://localhost:9001"
api_key = "secret"
```
