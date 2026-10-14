# HTTP Server

The {{ product_name }} Orchestrator provides its service through an HTTP server. It acts as a gateway for the
orchestrated services and forwards requests to the appropriate service instance.

## Configuration

The section in the [configuration file](README.md) is called `http`.

| Field        | Type                 | Required | Default value | Description                                                   |
| ------------ | -------------------- | -------- | ------------- | ------------------------------------------------------------- |
| `address`    | `string`             | no       | `0.0.0.0`     | IP address to which to listen for incoming connections        |
| `port`       | `uint`               | no       | `11222`       | TCP port number where the HTTP server can be reached          |
| `public_url` | `string`             | yes      | -             | The publicly reachable URL of the orchestrator                |
| `api_keys`   | Key/value pair array | yes      | -             | The API keys that authorize requests towards the orchestrator |

### Listening address

The `address` field must be an IPv4 or IPv6 address, hostnames are not supported.

- `0.0.0.0` binds to all IPv4 interfaces (default)
- `::` binds to all IPv6 interfaces. Depending on the operating system configuration this includes IPv4 connections.
- `127.0.0.1` or `::1` only accept connections from localhost

### Public URL

Clients reach the RoomServer signaling and the LiveKit endpoints through the orchestrator, so the `public_url` must
be reachable by clients.

When a RoomServer token is requested through the orchestrator, the orchestrator replaces the signaling URL in the
response with `<public_url>/roomserver/`. The LiveKit URL configured in the controller
(`roomserver.modules.livekit.public_url`) must point to `<public_url>/livekit/`.

When the orchestrator is served under a sub path behind a reverse proxy, the sub path must be included in the
`public_url`, e.g. `https://example.com/orchestrator/`.

### API Keys

The API keys authorize requests towards the `/register` and `/metrics` endpoints. See [Metrics](metrics.md) for other
authorization options of the `/metrics` endpoint. At least one API key must be configured and every key id must be
unique, otherwise the orchestrator fails to start.

An API key can be configured as string (`"<key_id>:<key_secret>"`) or as key/value pair
(`{ id = "<key_id>", secret = "<key_secret>" }`). Services that register with the orchestrator must use one of these
keys in their `orchestrator.api_key` setting. The controller is configured with one of these keys as well.

When set through the environment, the keys are provided as a comma-separated list:

```sh
OT_ORCHESTRATOR_HTTP__API_KEYS="orchestrator:secret,another_key:secret2"
```

### Example

```toml
[http]
address = "0.0.0.0"
port = 11222
public_url = "https://orchestrator.example.com/"
api_keys = [{ id = "orchestrator", secret = "secret" }]
```
