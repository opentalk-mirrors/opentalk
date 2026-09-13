# RoomServer

!!! warning

    The RoomServer is currently in a beta state.

The {{ product_name }} RoomServer provides signaling for meetings. When it is
enabled, the controller sends participants who are joining a meeting to the
RoomServer. The controller’s built-in signaling is disabled in this case.

## Configuration

The section in the [configuration file](configuration.md) is called
`roomserver`.

| Field                  | Type                                        | Required | Default value | Description                                                                                                                      |
| ---------------------- | ------------------------------------------- | -------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `kind`                 | `enum`                                      | yes      | -             | Whether the roomserver is running embedded in the controller process or as an external service. Either `internal` or `external`. |
| `service_url`          | `string`                                    | depends  | -             | The URL the controller uses for requests to the roomserver. Required when `kind = "external"`.                                   |
| `api_key`              | [API key](#api-key)                         | yes      | -             | API token used by the controller to authenticate against the RoomServer API.                                                     |
| `asset_storage`        | [Asset storage](#asset-storage)             | yes      | -             | Storage backend for room assets (e.g., meeting reports).                                                                         |
| `conference`           | [Conference](#conference)                   | no       | -             | Conference signaling settings. Only relevant when `kind = "internal"`.                                                           |
| `defaults`             | [Defaults](#defaults)                       | no       | -             | Default and fallback values. Only relevant when `kind = "internal"`.                                                             |
| `reports`              | [Reports](#reports)                         | no       | -             | Configuration related to report generation. Only relevant when `kind = "internal"`.                                              |
| `websocket_rate_limit` | [WebSocketRateLimit](#websocket-rate-limit) | no       | see below     | Websocket rate limit settings for the RoomServer.                                                                                |
| `room_idle_timeout`    | `uint`                                      | no       | 60            | The duration in seconds after which a room without participants is closed.                                                       |
| `modules`              | [Module settings](#modules)                 | yes      | -             | Enabled RoomServer modules and their settings.                                                                                   |

### API key

| Field    | Type     | Required | Default value | Description         |
| -------- | -------- | -------- | ------------- | ------------------- |
| `id`     | `string` | yes      | -             | API key identifier. |
| `secret` | `string` | yes      | -             | API key secret.     |

### Asset storage

The RoomServer stores assets such as meeting reports using one of the storage
backends below.

| Field    | Type     | Required | Default value | Description                                                                                        |
| -------- | -------- | -------- | ------------- | -------------------------------------------------------------------------------------------------- |
| `type`   | `string` | yes      | -             | Storage backend type. Supported values: `in_memory`, `controller`.                                 |
| `url`    | `string` | depends  | -             | Base URL of the controller’s asset endpoint. Required when `type = "controller"`.                  |
| `secret` | `string` | depends  | -             | Shared secret used to authenticate RoomServer asset requests. Required when `type = "controller"`. |

#### Asset storage types

- `in_memory`: Assets are stored in memory by the RoomServer and are lost on
  restart or the room being closed. Suitable for testing only.
- `controller`: Assets are stored via the controller’s asset endpoint. The
  RoomServer authenticates using the `secret` value.

### Conference

| Field               | Type     | Required | Default value                          | Description                                                                         |
| ------------------- | -------- | -------- | -------------------------------------- | ----------------------------------------------------------------------------------- |
| `signaling_salt`    | `string` | no       | random generated 24 character `string` | A random salt is generated at startup when not set. This value must be kept secret. |
| `room_idle_timeout` | `u64`    | no       | `60`                                   | The duration in seconds after which a room without participants is closed.          |

#### Signaling Salt

!!! danger

    This value must be kept secret.

The signaling salt is used to derive a participants device id from their device secret.
The device id is used as a unique identifier for a device.
When not set, a random 24 character string is generated at startup.
This means, that the signaling salt changes on each restart when not configured.

#### Room Idle Timeout

When all participants leave a room, it is not destroyed immediately.
This is to ensure that the state of the room is not lost, when no participant is present for a short time.
The room idle timeout determines how many seconds the RoomServer waits until destroying a room and dropping its state.

### Defaults

| Field                              | Type   | Required | Default value | Description                                                                                                                              |
| ---------------------------------- | ------ | -------- | ------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `screen_share_requires_permission` | `bool` | no       | `true`        | When true, participants can't share their screens unless permission is granted by a moderator. Moderators can always share their screen. |

### Reports

| Field   | Type            | Required | Default value | Description                                         |
| ------- | --------------- | -------- | ------------- | --------------------------------------------------- |
| `typst` | [Typst](#typst) | no       | -             | Configuration of typst, used for report generation. |

#### Typst

| Field           | Type   | Required | Default value               | Description                                  |
| --------------- | ------ | -------- | --------------------------- | -------------------------------------------- |
| `packages_path` | `Path` | no       | `/usr/share/typst/packages` | The location where typst looks for packages. |

#### Websocket Rate Limit

Rate limiting is enabled by default, to disable it, you have to set the `disabled` flag to `true`.
A missing configuration will fall back to the default values.
The implementation uses the token bucket algorithm. Each websocket message that is sent by a participant consumes one token.
A websocket connection has a maximum amount of tokens that can be available at a time (the token bucket).
Each second, the bucket is filled with a configured amount of tokens.
The algorithm allows the configuration to have a reasonably small amount of messages per second, while still allowing 'bursts' of messages until the tokens in the bucket are fully consumed.

| Field               | Type   | Required | Default value | Description                                                             |
| ------------------- | ------ | -------- | ------------- | ----------------------------------------------------------------------- |
| `disabled`          | `bool` | no       | `false`       | WebSocket rate limiting is disabled when set to `true`.                 |
| `tokens_per_second` | `uint` | no       | `10`          | The amount of messages that can be consistently sent by participants.   |
| `token_bucket_size` | `uint` | no       | `30`          | The maximum amount of tokens that can be held per websocket connection. |

### Modules

Module configuration is expressed under `roomserver.modules.<module>`. Only
modules listed in the configuration are enabled.

#### `automod`

Enables auto moderation support. This module has no additional settings.

#### `chat`

Enables chat support.

#### `rate_limit`

Optional rate-limiting for chat messages uses a token-bucket algorithm.

| Field                            | Type    | Required | Default value | Description                                                                                       |
| -------------------------------- | ------- | -------- | ------------- | ------------------------------------------------------------------------------------------------- |
| `rate_limit.tokens_per_second`   | `uint`  | yes      | -             | Tokens added per second.                                                                          |
| `rate_limit.token_bucket_size`   | `uint`  | yes      | -             | Maximum number of tokens in the bucket.                                                           |
| `rate_limit.slow_down_threshold` | `float` | yes      | -             | Proportion (0–1) of the limit at which clients receive an error message asking them to slow down. |

#### `e2ee`

Enables end-to-end encryption support. This module has no additional settings.

#### `echo`

Enables the echo module. This module has no additional settings.

#### `legal_vote`

Enables legal vote support. This module has no additional settings.

#### `livekit`

LiveKit signaling configuration for the RoomServer.

| Field         | Type     | Required | Default value | Description                                |
| ------------- | -------- | -------- | ------------- | ------------------------------------------ |
| `service_url` | `string` | yes      | -             | LiveKit URL reachable by the RoomServer.   |
| `public_url`  | `string` | yes      | -             | LiveKit URL reachable by clients.          |
| `api_key`     | `string` | yes      | -             | LiveKit API key used by the RoomServer.    |
| `api_secret`  | `string` | yes      | -             | LiveKit API secret used by the RoomServer. |

#### `meeting_notes`

Enables the meeting notes integration (Etherpad).

| Field      | Type     | Required | Default value | Description        |
| ---------- | -------- | -------- | ------------- | ------------------ |
| `base_url` | `string` | yes      | -             | Etherpad base URL. |
| `api_key`  | `string` | yes      | -             | Etherpad API key.  |

#### `meeting_report`

Enables meeting report generation support. This module has no additional settings.

#### `moderation`

Enables moderation support. This module has no additional settings.

#### `polls`

Enables polls support. This module has no additional settings.

#### `raise_hands`

Enables hand raise support. This module has no additional settings.

#### `shared_folder`

Shared folder integration. This module has no additional settings.

#### `subroom_audio`

Enables subroom audio support. This module has no additional settings.

#### `timer`

Enables timer support. This module has no additional settings.

#### `training_participation_report`

Enables training participation report support. This module has no additional settings.

#### `whiteboard`

Enables the whiteboard integration (Spacedeck).

| Field      | Type     | Required | Default value | Description         |
| ---------- | -------- | -------- | ------------- | ------------------- |
| `base_url` | `string` | yes      | -             | Spacedeck base URL. |
| `api_key`  | `string` | yes      | -             | Spacedeck API key.  |

## Examples

### Minimal RoomServer configuration

```toml
[roomserver]
url = "http://localhost:11333"
api_key = { id = "controller", secret = "secret" }

[roomserver.asset_storage]
type = "in_memory"

[roomserver.modules]
```

### Controller-backed asset storage with modules

```toml
[roomserver]
url = "https://roomserver.example.com"
api_key = { id = "controller", secret = "secret" }

[roomserver.asset_storage]
type = "controller"
url = "https://controller.example.com"
secret = "secret"

[roomserver.modules.chat]
[roomserver.modules.chat.rate_limit]
tokens_per_second = 3
token_bucket_size = 10
slow_down_threshold = 0.8

[roomserver.modules.e2ee]

[roomserver.modules.livekit]
api_key = "devkey"
api_secret = "secret"
public_url = "https://livekit.example.com"
service_url = "https://livekit.internal.example.com"

[roomserver.modules.echo]
[roomserver.modules.polls]
[roomserver.modules.timer]

[roomserver.modules.meeting_notes]
api_key = "secret"
base_url = "https://etherpad.example.com"
```
