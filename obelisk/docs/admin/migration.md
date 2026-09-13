---
title: Migration Guide
---

# Migration Guide

This guide describes breaking changes to the configuration file and how to
update an existing settings file between releases.

## `v0.21` → `v0.22`

Release `0.22` switches from OIDC/Keycloak authentication against the controller
to API-key authentication, and addresses the controller by a full URL instead of
a bare domain plus an `insecure` flag. These are **breaking** configuration
changes.

### Overview

| Section        | `v0.21`                                | `v0.22`             | Action                   |
| -------------- | -------------------------------------- | ------------------- | ------------------------ |
| `[auth]`       | `issuer`, `client_id`, `client_secret` | *removed*           | Delete the whole section |
| `[controller]` | `domain`, `insecure`                   | `url`, `api_key`    | Rewrite (see below)      |
| `[sip]`        | `max_video_bitrate` in kbit/s          | in bit/s            | Multiply value by 1000   |

Unknown configuration keys are now logged as warnings on startup instead of
being silently ignored. A new optional SIP setting (`offer_srtp`) was also
added; see the [configuration reference](configuration.md).

### 1. Remove `[auth]`

API-key authentication replaces Keycloak/OIDC. Delete the entire section:

```toml
# DELETE
[auth]
issuer = "http://localhost/auth/realms/Example"
client_id = "Obelisk"
client_secret = "obeliskclientsecret"
```

### 2. Rework `[controller]`

The controller is now addressed by a full URL (with scheme) instead of a bare
domain plus an `insecure` flag, and authentication uses an API key instead of
OIDC.

```toml
# BEFORE (v0.21)
[controller]
domain = "localhost:8000"
#insecure = false

# AFTER (v0.22)
[controller]
url = "http://localhost:8000"
api_key = { id = "obelisk", secret = "examplesecret" }
```

Mapping of the old transport-security flag:

- `insecure = true`  → `url = "http://…"`
- `insecure = false` → `url = "https://…"`

The `api_key` identifies obelisk against the controller. It consists of an `id`
and a `secret`. The controller must be configured with a matching key so it can
verify obelisk's requests. You can provide it either as a table or as a single
`"<id>:<secret>"` string:

```toml
# Table form
api_key = { id = "obelisk", secret = "examplesecret" }

# Equivalent string form
api_key = "obelisk:examplesecret"
```

### 3. Convert `max_video_bitrate` to bit/s

The unit of `max_video_bitrate` changed from kbit/s to bit/s. Multiply the
previous value by 1000.

```toml
# BEFORE (v0.21)
[sip]
#max_video_bitrate = 6000

# AFTER (v0.22)
[sip]
#max_video_bitrate = 6000000
```

### 4. Review warnings on startup

Unknown configuration keys are now reported on startup, for example:

```text
WARNING: Unknown configuration key auth.issuer
```

If you see such warnings, remove or rename the reported keys.
