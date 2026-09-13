# Migration Guide

This guide describes breaking changes to the configuration file and how to
update an existing settings file between releases.

## `v0.17` -> `v0.18`

There are no breaking changes between `v0.17` and `v0.18`. No migration is necessary.

## `v0.16` → `v0.17`

Release `0.17` replaces RabbitMQ-based job delivery with a REST/HTTP model and
switches from OIDC/Keycloak authentication to API-key authentication. These are
**breaking** configuration changes.

### Overview

| Section          | `v0.16`                                  | `v0.17`                              | Action                       |
| ---------------- | ---------------------------------------- | ------------------------------------ | ---------------------------- |
| `[auth]`         | `issuer`, `client_id`, `client_secret`   | *removed*                            | Delete the whole section     |
| `[rabbitmq]`     | `uri`, `queue`                           | *removed*                            | Delete the whole section     |
| `[controller]`   | `domain`, `insecure`                     | `url`, `api_key`, `upload_chunk_size`| Rewrite (see below)          |
| `[http]`         | *did not exist*                          | `addr`, `port`, `api_keys`           | Add (required)               |
| `[orchestrator]` | *did not exist*                          | `url`, `api_key`                     | Add (optional)               |

### 1. Remove `[auth]`

API-key authentication replaces Keycloak/OIDC. Delete the entire section:

```toml
# DELETE
[auth]
issuer = "http://localhost:8080/auth/realms/MyRealm"
client_id = "Recorder"
client_secret = "INSERT_KEY"
```

### 2. Remove `[rabbitmq]`

Jobs are now delivered over HTTP (see the new `[http]` section). Delete the
entire section:

```toml
# DELETE
[rabbitmq]
uri = "amqp://username:password@localhost/%2F"
queue = "recorder"
```

### 3. Rework `[controller]`

The controller is now addressed by a full URL (with scheme) instead of a bare
domain plus an `insecure` flag, and authentication uses an API key instead of
OIDC.

```toml
# BEFORE (v0.16)
[controller]
domain = "localhost:11311"
insecure = true

# AFTER (v0.17.1)
[controller]
url = "http://localhost:11311"
api_key = { id = "recorder", secret = "secret" }
```

Mapping of the old transport-security flag:

- `insecure = true`  → `url = "http://…"`
- `insecure = false` → `url = "https://…"`

The `api_key` identifies the recorder against the controller. It consists of an
`id` and a `secret`, both of which must be non-empty. The controller must be
configured with a matching key so it can verify the recorder's requests. You can
provide it either as a table (like in the example above) or as a single
`"<id>:<secret>"` string:

```toml
# Table form
api_key = { id = "recorder", secret = "secret" }

# Equivalent string form
api_key = "recorder:secret"
```

!!! info "How the API key is used"

    The recorder does not send the `secret` over the wire. Instead it uses the
    `secret` to sign a short-lived (60 second) `HS256` JSON Web Token per
    request, with the `id` set as the token's key id (`kid`).

### 4. Add `[http]` (required)

The recorder now runs its own HTTP server that other services call into. The
`api_keys` field is required.

```toml
[http]
addr = "0.0.0.0"   # optional, default 0.0.0.0
port = 11511       # optional, default 11511
# API keys for internal service endpoints; each entry is a string
# ("<id>:<secret>") or a table ({ id = "…", secret = "…" }).
api_keys = [{ id = "controller", secret = "secret" }]
```

### 5. Add `[orchestrator]` (optional)

Only required when the orchestrator is used:

```toml
[orchestrator]
url = "http://127.0.0.1:11222"
api_key = { id = "recorder", secret = "secret" }
```
