# Metrics

The {{ product_name }} Orchestrator serves a `/metrics` endpoint on its [HTTP server](http.md). The endpoint returns a
JSON document that lists the registered RoomServer, Recorder and Transcription instances together with their current
state.

Access to the endpoint can be restricted in two ways, which can be combined:

- Token authorization through the `Authorization` header (enabled by default)
- An allowlist of client IP addresses and networks

## Configuration

The section in the [configuration file](README.md) is called `metrics`. The section is optional. Without it, the
endpoint requires a JWT based on one of the orchestrator's [API keys](http.md#api-keys).

| Field                | Type       | Required | Default value | Description                                                          |
| -------------------- | ---------- | -------- | ------------- | -------------------------------------------------------------------- |
| `disable_token_auth` | `bool`     | no       | `false`       | Disable token authorization for the `/metrics` endpoint              |
| `bearer_token`       | `string`   | no       | -             | Static token that is expected instead of a JWT based on the API keys |
| `allowlist`          | `string[]` | no       | -             | IP addresses and networks that may access the `/metrics` endpoint    |

### Token authorization

By default, requests must contain an `Authorization: Bearer <jwt>` header. The JWT is signed with one of the keys
configured in `http.api_keys`, the same way the orchestrated services authorize towards the orchestrator.

When `bearer_token` is set, the endpoint expects the configured token instead:

```text
Authorization: Bearer <bearer_token>
```

This is useful for monitoring systems like Prometheus that can only send a static token.

When `disable_token_auth` is `true`, no token is required. `bearer_token` is ignored in this case. Only disable token
authorization when access is restricted otherwise, e.g. through the `allowlist` or because the endpoint is only
reachable from an internal network.

Requests without a valid token are rejected with `401 Unauthorized`.

### IP address allowlist

When `allowlist` is set, only clients whose IP address is contained in one of the entries can access the endpoint. The
allowlist applies independently of the token authorization, so a client needs both an allowed IP address and a valid
token unless `disable_token_auth` is `true`. Requests from other IP addresses are rejected with `403 Forbidden`.

| `allowlist` value                | Behavior                                  |
| -------------------------------- | ----------------------------------------- |
| not set                          | No IP address restriction                 |
| `[]`                             | Every request is denied                   |
| `["10.0.0.0/8", "192.168.1.2"]`  | Only clients from these networks or hosts |
| `["0.0.0.0/0", "::/0"]`          | Every IPv4 and IPv6 address               |

Each entry is an IPv4 or IPv6 address with an optional prefix length:

- An address without a prefix length matches exactly that address, e.g. `192.168.1.2` or `fd00::1`.
- An address with a prefix length matches the whole network. Host bits are ignored, `10.1.2.3/8` is treated like
  `10.0.0.0/8`.

When the HTTP server listens on `::` and also accepts IPv4 connections, IPv4 clients are matched against the IPv4
entries of the allowlist as usual. Entries in the IPv4-mapped IPv6 notation (e.g. `::ffff:10.0.0.0/104`) never match
and must be written as IPv4 networks instead.

#### Reverse proxies

The allowlist is checked against the address of the TCP connection. When the orchestrator runs behind a reverse proxy
or a Kubernetes ingress, this is the address of the proxy, not of the original client. Headers like `X-Forwarded-For`
are not evaluated. In this setup, the allowlist effectively only restricts which proxies can reach the endpoint, so
access to `/metrics` should additionally be restricted at the proxy or by token authorization.

### Examples

Use a static bearer token, e.g. for a Prometheus scrape job:

```toml
[metrics]
bearer_token = "my-token"
```

Only allow access from an internal network without any token:

```toml
[metrics]
disable_token_auth = true
allowlist = ["10.0.0.0/8", "fd00::/8"]
```

Require a JWT based on the API keys and restrict access to a single monitoring host:

```toml
[metrics]
allowlist = ["192.168.1.2"]
```
