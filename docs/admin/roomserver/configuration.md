---
title: Configuration
---

# Configuring {{ product_name }} RoomServer

The {{ product_name }} RoomServer can be configured using environment variables and a configuration file.
Environment variables take precedence over the configuration file.

## Environment variables

Settings in the configuration file can be overwritten by environment variables,
nested fields are separated by two underscores `__`. The pattern looks like
this:

```sh
OPENTALK_ROOM_<field>__<nested-field>…
```

### Arrays

Some configuration options allow to configure multiple values.

- `OPENTALK_ROOM_METRICS__ALLOWLIST` expects a coma separated list
    - `172.0.0.0/8,192.168.0.0/16`
- `OPENTALK_ROOM_HTTP__API_KEYS` expects a coma separated list where key and secret are separated by a colon `:`
    - `roomserver:secret1,recorder:secret2`

## Sections in the configuration file

- [Http](http_server.md)
- [Controller](controller.md)
- [Monitoring](observability/monitoring.md)
- [Metrics](observability/metrics.md)
- [Tracing](observability/tracing.md)
- [Conference](conference.md)
- [Defaults](defaults.md)
- [Reports](reports.md)
- [Internal](internal.md)

## Example configuration file

This file can be found in the source code distribution under `example/roomserver.toml`

<!-- begin:fromfile:example/roomserver.toml.md -->

```toml
[http]
# HTTP server settings

# The IPv4/IPv6 address to bind the HTTP server to.
# Binds to all interfaces by default:
#address = "0.0.0.0"

# All IPv6 interfaces:
#address = "::"

# Localhost only:
#address = "127.0.0.1"
#address = "::1"

# The URL of the roomserver that is reachable by internal services
# service_url = "http://localhost:13333"

# The publicly reachable URL of this server
public_url = "http://localhost:11333"

# The api keys for internal service endpoints
#
# The roomserver can have multiple api keys configured. An api key can be configured as string ("<key_id>:<key_secret>")
# or as key/value pair ({id = "<key_id>", secret = "<key_secret>"}).
api_keys = [{ id = "roomserver", secret = "secret" }]

# The port to bind the HTTP server to (defaults to 11333)
#port = 11333

# Disable the OpenAPI endpoint under `/docs/openapi.json` and the corresponding
# swagger endpoint under `/swagger`.
#enable_openapi = false

[controller]
# Controller settings
#
# If not configured, assets and module resources will be saved in memory.

# The URL of the controller
url = "http://localhost:8000"

# The api key for the controllers service API
api_key = { "id" = "controller", "secret" = "secret" }

[monitoring]
# Monitoring settings

# Monitoring is optional and disabled by default
#addr = "0.0.0.0"
#port = 11411

[metrics]
# Metrics settings

# Metrics is optional and disabled by default
#port = 11412
# IP addresses allowed to access the metrics endpoint
#allowlist = []

# Allow access from localhost
#allowlist = ["127.0.0.0/24", "::ffff:0:0/96"]

#[tracing]
# Tracing settings

# Tracing is optional and disabled by default
#otlp_tracing_endpoint = "http://localhost:4317"

# Conference settings are optional
#[conference]
# Conference settings

# Used to derive participant ids from device secrets.
# A random salt is generated at startup when not set. This value must be kept secret.
# signaling_salt = "random string at least 24 characters long"

# [defaults]
# When true, participants can't share their screens unless permission is granted by a moderator.
# Moderators can always share their screen.
# screen_share_requires_permission = true

#[reports.typst]
# The location where typst looks for packages.
#packages_path = "/usr/share/typst/packages"

# Optional orchestrator configuration
#[orchestrator]
# If configured, the roomserver will register at the orchestrator when started.
#
# If the `http.service_url` is configured, the roomserver will include that URL in the orchestrators registration
# request. If not, the roomserver will only provide its `http.port` in the registration and the orchestrator must build
# the roomservers URL by using the received websocket address and the provided port.
#
# The API key of the orchestrator
#api_key = { id = "orchestrator", secret = "secret" }
# The orchestrator URL
#url = "http://127.0.0.1:11222"

# Internal settings are optional and use default values if not set.
# [internal]
# The number of rooms to modify in parallel when receiving a storage quota POST request.
# Higher numbers will reduce the time it takes to update the storage quota for all rooms,
# but will increase server load while the requests are running.
# parallel_storage_quota_requests = 5
```

<!-- end:fromfile:example/roomserver.toml.md -->
