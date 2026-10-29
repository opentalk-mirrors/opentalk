---
title: Migration Guide
---

# Migrating to {{ product_name }} 26.3 { #migrating-to-26-3 }

{{ product_name }} 26.3 builds on the architecture introduced with [26.2](#migrating-to-26-2). There are no
changes to the overall architecture of a deployment, but there are a few changes that require attention from
administrators:

- The bundled **PostgreSQL** is upgraded to a new major version (`17` → `18`).
- Signaling metrics are available again and are now provided by the **RoomServer**.
- Invite codes were removed in favor of a single join link per room.
- The **Orchestrator** is now scalable when a redis storage backend is configured.

This guide covers the upgrade from {{ product_name }} 26.2 to 26.3. It summarizes what changed and links to the
per-component migration guides for the details.

## Component versions

| Component      | 26.2.x      | 26.3.0      | Migration notes                                         |
| -------------- | ----------- | ----------- | ------------------------------------------------------- |
| Controller     | `v0.34.x`   | `v0.35.3`   | [Controller `v0.35.0`](controller/migration/v0.35.0.md) |
| RoomServer     | `v0.1.x`    | `v0.10.3`   | See [below](#2-update-the-controller-and-roomserver)    |
| Orchestrator   | `v0.8.x`    | `v0.12.1`   | See [below](#3-update-the-orchestrator)                 |
| Recorder       | `v0.17.x`   | `v0.18.2`   | See [below](#4-update-the-recorder)                     |
| Obelisk        | `v0.22.x`   | `v0.23.2`   | no configuration changes                                |
| SMTP Mailer    | `v0.18.x`   | `v0.19.1`   | See [below](#5-update-the-smtp-mailer)                  |
| Web Frontend   | `v2.9.x`    | `v2.10.2`   | -                                                       |
| Outlook Add-in | `v0.4.x`    | `v0.6.0`    | -                                                       |
| LiveKit Server | `v1.13.5`   | `v1.13.10`  | -                                                       |
| Keycloak       | `v26.7.2`   | `v26.8.1-1` | -                                                       |
| PostgreSQL     | `v17.11.0`  | `v18.6.0`   | **Major upgrade**, see [below](#1-upgrade-postgresql)   |
| NDT Server     | `v0.25.3-1` | `v0.26.1`   | -                                                       |

!!! note
    The RoomServer crates switched to unified versioning, which is why the service version jumps from `v0.1.x` to
    `v0.10.x`. There are no intermediate RoomServer releases to migrate through.

## Unchanged components

| Component | Version                          |
| --------- | -------------------------------- |
| RabbitMQ  | `v4.3.7`                         |
| Redis     | `v8.10.2`                        |
| Minio     | `RELEASE.2025-10-15T17-29-55Z-1` |
| Garage    | `v2.3.0-1`                       |
| Traefik   | `v3.7.16`                        |

## What changed

### One join link per room

Invite codes were removed from all components. Rooms are now joined through a single join link, and the room's
access settings (password, waiting room, guest access) decide who can enter. The invite-code API was removed from the
Controller, and the Web Frontend, SMTP Mailer, Outlook Add-in, Obelisk and Recorder were updated accordingly.

Invitation links sent before the change that still use the legacy URL format continue to work.

### Room aliases

Rooms can now have an optional alias to give a personal room a readable name. To protect rooms against brute-force
attacks, a random suffix is appended to the alias by default. The behavior can be tuned in the Controller,
see [below](#2-update-the-controller-and-roomserver).

### Further changes

- Guests can join rooms with 'very high level of protection' turned on.
- When a event/room is deleted through the REST API, any associated running meeting is closed.
- The Outlook Add-in supports recurring meetings.

## 1. Upgrade PostgreSQL

26.3 upgrades PostgreSQL from `17` to `18`. PostgreSQL does not read data directories of a previous major version, so
replacing the container image alone is **not** sufficient. The data has to be migrated, either with a dump and restore
or with `pg_upgrade`.

For **Opentalk compose** deployments follow the update procedure in
[Updating OpenTalk Compose](deployment/opentalk-compose/1-quickstart-installation.md#7-updating-opentalk-compose).

For other deployments, follow the
[PostgreSQL upgrade documentation](https://www.postgresql.org/docs/18/upgrading.html).

## 2. Update the Controller and RoomServer

Controller `v0.35` requires RoomServer `v0.10`. Running a `v0.35` Controller with a RoomServer from 26.2 is not
supported.

With `roomserver.kind = "internal"`, the RoomServer is part of the Controller and there is no separate roomserver update
required.

With `roomserver.kind = "external"`, the RoomServer must be updated to `v0.10` to ensure API compatibility. There are
**no** breaking changes in the RoomServer configuration.

See the [Controller `v0.35.0` migration guide](controller/migration/v0.35.0.md) for all details.

### Room alias configuration

The new `[defaults.room_alias]` section in the Controller configuration is optional. Existing configurations keep
working with secure defaults:

```toml
[defaults.room_alias]
# When `true`, rooms with an alias will not have a suffix appended to their name.
# This makes the rooms vulnerable to brute-force or guessing attacks. Only enable it in trusted environments.
disable_suffix = false
# Length of the random suffix appended to an aliased room name.
# Allowed values are between 8 and 64, inclusive.
suffix_length = 16
```

### Signaling metrics

With 26.3 The opentelemetry `signaling_*` metrics moved from the Controller's `/metrics` endpoint to the
RoomServer `/metrics` endpoint. When configuring the RoomServer to be `roomserver.kind = "internal"`, the `/metrics`
endpoint is still served by the Controller

Some metrics were removed and a few were renamed. Check out the
[Controller `v0.35.0` migration guide](controller/migration/v0.35.0.md#metric-mapping) for the full mapping and
the [RoomServer metrics documentation](roomserver/observability/metrics.md) for all available metrics.

## 3. Update the Orchestrator

Orchestrator `v0.12` gets a mandatory `storage` configuration and now supports `redis` as storage backend. The redis
storage backend enables the Orchestrator to scale. With `local` storage the state is kept in the memory of the
orchestrator process and only one Orchestrator instance can be deployed.

When only running a single orchestrator, add the `storage` section and set the kind to `"local"` in the
`orchestrator.toml`:

```toml
[storage]
kind = "local"
```

### Scaling with redis

When using redis as a storage backend, multiple Orchestrator instance can run in parallel and share their state through
redis. For this to work, the Orchestrator instances need to be deployed behind a load balancer.

Requirements for the Redis instance:

- Redis 7.2 or newer. The orchestrator always connects with the RESP3 protocol.
- Keyspace notifications for keyspace events (`K`) and expired events (`x`). The orchestrator reads the
  `notify-keyspace-events` setting on startup and enables missing flags with `CONFIG SET`. If the `CONFIG` command
  isn't permitted for the orchestrator, e.g. on managed Redis offerings, enable the flags (`notify-keyspace-events Kx`)
  in the Redis configuration beforehand.
- Only single instance Redis deployments. Redis clusters are currently **not** supported.

When running multiple Orchestrator instances, add the `storage` section, set the kind to `"redis"` and add the redis
`url` in the `orchestrator.toml`:

```toml
[storage]
# Set the storage kind to redis
kind = "redis"

# The url of the redis instance. Can only be set when kind = "redis"
url = "redis://localhost:6379"
```

!!! note
    Make sure that all orchestrator instances are behind a load balancer and accessible through the same URL.

## 4. Update the Recorder

Recorder `v0.18` has no breaking configuration changes.

Environment variables with a double underscore after the `OPENTALK_REC` prefix are deprecated. Use a single underscore
after the prefix instead. Nested fields are still separated by two underscores:

```sh
# deprecated
OPENTALK_REC__CONTROLLER__URL=http://localhost:11311
# correct
OPENTALK_REC_CONTROLLER__URL=http://localhost:11311
```

The deprecated form keeps working for now, but the Recorder prints a warning on startup. See the
[Recorder configuration](recorder/configuration/README.md#environment-variables) for details.

## 5. Update the SMTP Mailer

With the removal of invite codes, the `guest_link_builder` option in the `[template-builder]` section was removed. If your
configuration still sets it, the SMTP Mailer logs an `Unknown configuration key` warning on startup. Remove the option
from your configuration:

```toml
[template-builder]
#guest_link_builder = "{base_url}/invite/{invite_id}"
```

## 6. Update the Web Frontend and the remaining services

The following components have **no** configuration changes and can be updated by replacing the container image or
package:

- Web Frontend `v2.10.2`
- Obelisk `v0.23.2`
- Outlook Add-in `v0.6.0`
- LiveKit Server `v1.13.10`
- Keycloak `v26.8.1-1`

# Migrating to {{ product_name }} 26.2 { #migrating-to-26-2 }

{{ product_name }} 26.2 changes the architecture of a deployment. Meeting signaling moves out of the {{ product_name }}
Controller into the dedicated [RoomServer](roomserver/README.md), and the new {{ product_name }}
[Orchestrator](orchestrator/README.md) coordinates RoomServer and [Recorder](recorder/README.md) instances. The Recorder
no longer receives jobs through RabbitMQ. Instead, it gets them over HTTP.

With the **RoomServer** and **Orchestrator**, we plan to improve the stability and scalability of {{ product_name }}
deployments.

This guide covers the upgrade from {{ product_name }} 26.1 to 26.2. It summarizes what changed and links to the
per-component migration guides for the details.

## RoomServer introduction

With `26.2` the **RoomServer** replaces the signaling backend that previously ran on the **Controller**. It can be
deployed as a separate scalable service or as part of the **Controller** process, depending on the deployment's
requirements.

Having signaling in another process allows us to scale the signaling and REST API independently. The Controllers only
concern now is the web API, storage API and persistent data. All real-time logic moved to the roomserver. Previously in
the controller, each participant had their own room process and the meeting state was synced through redis. This
resulted in redis round trips, redis locks and a lot of complexity.

The RoomServer has a simpler, more robust meeting state. Each RoomServer can hold a number of rooms, each room is a
self contained task and the meeting state is held in local memory. This simplifies module development and we can build
more reliable meeting rooms.

The full RoomServer documentation can be found [here](roomserver/README.md).

## Orchestrator introduction

With `26.2` an {{ product_name }} deployment can now have an Orchestrator. This component is optional and its purpose is
to scale other services like the **Roomserver** or **Recorder**. A small deployment with only single instance services
does not gain any benefits from running the Orchestrator.

The Orchestrator keeps track of resources that are assigned to specific service instances. In case of the RoomServer, it
keeps track which instance holds which meeting room (room id). If it receives a request that is associated with a room
id, it will forward that request to the appropriate instance. When the request refers to an unknown room id, the
Orchestrator will select the RoomServer instance with the lowest load for the new meeting room. The same behaviour
applies when scaling the Recorder.

The full Orchestrator documentation can be found [here](orchestrator/README.md).

## Component versions

| Component      | 26.1      | 26.2.0    | Migration notes                                         |
| -------------- | --------- | --------- | ------------------------------------------------------- |
| Controller     | `v0.33.x` | `v0.34.4` | [Controller `v0.34.0`](controller/migration/v0.34.0.md) |
| RoomServer     | -         | `v0.1.3`  | New component, see [below](#3-deploy-the-roomserver)    |
| Orchestrator   | -         | `v0.8.1`  | New component, see [below](#1-deploy-the-orchestrator)  |
| Recorder       | `v0.16.x` | `v0.17.1` | See [below](#4-update-the-recorder)                     |
| Obelisk        | `v0.21.x` | `v0.22.2` | [Obelisk `v0.21` → `v0.22`](obelisk/migration.md)       |
| SMTP Mailer    | `v0.17.0` | `v0.18.0` | -                                                       |
| Web Frontend   | `v2.8.x`  | `v2.9.2`  | -                                                       |
| LiveKit Server | `v1.9.12` | `v1.13.5` | -                                                       |

## What changed

### Before (26.1)

- The **Controller** served the web API and handled meeting signaling in-process.
- The **Recorder** took recording jobs from a RabbitMQ queue (`recording_task_queue`) and authenticated against the
  Controller through Keycloak (OIDC client credentials).
- **Obelisk** authenticated against the Controller through Keycloak.

### After (26.2)

When the Orchestrator is configured, the Controller and the Web Frontend reach the RoomServers and Recorders through
the Orchestrator instead of connecting to them directly.

- The **Controller** serves the web API and storage API. It sends participants joining a meeting to a RoomServer.
- The **RoomServer** handles all meeting signaling and the meeting modules (chat, polls, whiteboard, meeting notes,
  recording, and so on). It stores assets through the Controller's service API.
- The **Orchestrator** is an optional registry for RoomServer and Recorder instances. When configured, it allows scaling
  the RoomServer and Recorder.
- The **Recorder** exposes an HTTP API. The RoomServer starts recordings through it. RabbitMQ and Keycloak are no longer
  used by the Recorder.
- **Obelisk** and the **Recorder** authenticate against the Controller with API keys instead of Keycloak clients.

## Upgrade order

Upgrade the components in this order:

1. Orchestrator (new)
1. Controller
1. RoomServer (new)
1. Recorder
1. Obelisk
1. Web Frontend and the remaining services

## 1. Deploy the Orchestrator

An Orchestrator is **required** when the deployment runs multiple instances of the RoomServer **or** the Recorder. When
these services are not scaled, the Orchestrator can be skipped.

A minimal orchestrator config looks like this:

```toml
[http]
# The public orchestrator url, must be reachable by clients
public_url = "<public-orchestrator-url>"
# The orchestrators api key, used to authorize requests towards the orchestrator
api_keys = [{ id = "orchestrator", secret = "<orchestrator-secret>" }]

[services]
# All api keys of proxied/scaled services
#
# These must match the keys in `http.api_keys` in the respective service
keys = [
   { id = "roomserver", secret = "<roomserver-secret>" },
   { id = "recorder", secret = "<recorder-secret>" },
]
```

The rest of the Orchestrator configuration is done in the controller or the RoomServer/Recorder. Each of these
components will have an orchestrator section that describes the necessary configuration changes.

See the [Orchestrator documentation](orchestrator/README.md) for more information.

## 2. Update the Controller

Controller `v0.34` reworks the configuration file. Follow the
[Controller `v0.34.0` migration guide](controller/migration/v0.34.0.md) for the full list of changes. In short:

- The `[roomserver]` section is now **required** and has a mandatory `kind` field. Set `kind = "external"` to use a
  dedicated RoomServer instance, set it to `kind = "internal"` to run it directly as a Controller process.
- The `url` key in `[roomserver]` was renamed to `service_url`.
- `[livekit]`, `[etherpad]`, `[spacedeck]` and `[subroom_audio]` moved under `[roomserver.modules.*]`. The `[authz]` and
  `[websocket_rate_limit]` tables were removed.
- `[http] service_api_keys` is now **required**. Add a key for every service that calls the Controller: RoomServer,
  Recorder and Obelisk.
- The `acl` and `fix-acl` CLI subcommands were removed.

A minimal Controller configuration for an external RoomServer without the orchestrator:

```toml
[roomserver]
kind = "external"
# The URL the controller uses to reach the roomserver
service_url = "http://roomserver:11333"
# Must match an entry in the roomserver's `[http] api_keys`
api_key = { id = "roomserver", secret = "<roomserver-secret>" }

[roomserver.modules.livekit]
api_key = "your-livekit-api-key"
api_secret = "your-livekit-api-secret"
# LiveKit signaling must proxied by the roomserver (or the orchestrator)
public_url = "<public-roomserver-url>/livekit/"
service_url = "http://localhost:7880"

[roomserver.modules.echo]
[roomserver.modules.chat]
[roomserver.modules.moderation]
# ... further modules, see the controller migration guide
```

To enable recordings, configure the `recording` module so that the RoomServer can reach the Recorder:

```toml
[roomserver.modules.recording]
url = "http://recorder:11511"
# Must match an entry in the recorder's `[http] api_keys`
api_key = { id = "recorder", secret = "<recorder-secret>" }
```

The `recording_task_queue` setting in the `[rabbit_mq]` section is no longer used by the Recorder and can be removed.

### Orchestrator { #controller-orchestrator }

The orchestrators proxies the service APIs of the Recorder and Roomserver. This means that the service URLs in the
Controllers configuration file must point to the respective Orchestrators proxy endpoint instead of the service:

| Service                          | Orchestrator endpoint |
| -------------------------------- | --------------------- |
| {{ product_name }} RoomServer    | `/roomserver/`        |
| {{ product_name }} Recorder      | `/recording/`         |

To scale the roomserver with the orchestrator:

```toml
[roomserver]
kind = "external"
# The orchestrators roomserver proxy URL, mind the trailing slash
service_url = "http://orchestrator:11222/roomserver/"
# The orchestrators api key
api_key = { id = "orchestrator", secret = "<orchestrator-secret>" }

[roomserver.modules.livekit]
# This is the orchestrators livekit proxy URL, mind the trailing slash
public_url = "http://orchestrator:11222/livekit/"
# ... the other LiveKit settings can stay as they are
```

!!! important
    When proxying the roomserver, the `public_url` of LiveKit needs to be the orchestrators LiveKit proxy url.

To scale the recorder:

```toml
[roomserver.modules.recording]
# The orchestrators roomserver proxy URL, mind the trailing slash
url = "http://orchestrator:11222/recording/"
# Must match an entry in the recorder's `[http] api_keys`
api_key = { id = "recorder", secret = "<recorder-secret>" }
```

## 3. Deploy the RoomServer

The RoomServer is a separate service with its own configuration file. The full reference is in
[Configuring the RoomServer](roomserver/configuration.md).

```toml
[http]
# The publicly reachable URL of the roomserver (used by the web frontend when not running the orchestrator)
public_url = "https://example.com/roomserver"
# The URL internal services use to reach the roomserver.
service_url = "http://roomserver:11333"
# Keys accepted by the roomserver
api_keys = [{ id = "roomserver", secret = "<roomserver-secret>" }]

[controller]
url = "http://controller:11311"
api_key = { id = "controller", secret = "<controller-secret>" }

[conference]
# Keep this value secret and stable across restarts and identical on all roomservers.
signaling_salt = "<random string at least 24 characters long>"

[metrics]
port = 11412
allowlist = ["<prometheus-ip>/32"]
```

!!! note
    Without the Orchestrator, the web frontend connects directly to the RoomServer's `public_url` over WebSocket. Update
    your reverse proxy configuration accordingly.

### Orchestrator

Add the orchestrator section to the `roomserver.toml`:

```toml
[orchestrator]
url = "http://orchestrator:11222"
api_key = { id = "orchestrator", secret = "<orchestrator-secret>" }
```

Add the roomservers API keys to the `services.keys` section in the `orchestrator.toml`.

If not done already, update the Controller configuration as described in the Controllers
[Orchestrator section](#controller-orchestrator) above.

## 4. Update the Recorder

Recorder `v0.17` replaces RabbitMQ with an HTTP API, and replaces Keycloak with API-key authentication. These are
**breaking** configuration changes.

| Section          | `v0.16`                                | `v0.17`                    | Action                   |
| ---------------- | -------------------------------------- | -------------------------- | ------------------------ |
| `[auth]`         | `issuer`, `client_id`, `client_secret` | _removed_                  | Delete the whole section |
| `[rabbitmq]`     | `uri`, `queue`                         | _removed_                  | Delete the whole section |
| `[controller]`   | `domain`, `insecure`                   | `url`, `api_key`           | Rewrite (see below)      |
| `[http]`         | -                                      | `addr`, `port`, `api_keys` | Add (required)           |
| `[orchestrator]` | -                                      | `url`, `api_key`           | Add                      |

Unknown configuration keys are now logged as warnings on startup.

A minimal Recorder configuration:

```toml
[controller]
# `insecure = true` → `http://…`, `insecure = false` → `https://…`
url = "https://controller"
# Must match an entry in the controller's `[http] service_api_keys`
api_key = { id = "controller", secret = "<controller-secret>" }

[http]
#addr = "0.0.0.0"
#port = 11511
# The Recorders 's API key
api_keys = [{ id = "recorder", secret = "<recorder-secret>" }]
```

The RoomServer (or Orchestrator in an Orchestrator deployment) must be able to reach the Recorder's HTTP port. For a
[Recorder on a separate host](deployment/opentalk-compose/5-advanced-installation-scenarios.md#3-external-opentalk-recorder-only-nodes),
you no longer need to expose RabbitMQ to it. Instead, open the Recorder's HTTP port (default `11511/tcp`) to the
RoomServer, and make sure the Recorder can reach the Controller, the Orchestrator and LiveKit.

Once the Recorder is migrated, you can delete the `Recorder` client and the `opentalk-recorder` role in Keycloak.

### Orchestrator

Add the orchestrator section to the `recorder.toml`:

```toml
[orchestrator]
url = "http://orchestrator:11222"
api_key = { id = "orchestrator", secret = "<orchestrator-secret>" }
```

Add the recorders API keys to the `services.keys` section in the `orchestrator.toml`.

If not done already, update the Controller configuration as described in the Controllers
[Orchestrator section](#controller-orchestrator) above.

## 5. Update Obelisk

Obelisk `v0.22` also switches from Keycloak to API-key authentication against the Controller and changes the unit of
`max_video_bitrate`. Follow the [Obelisk migration guide](obelisk/migration.md).
