# Recording

## Overview

The recording module allows for the recording / streaming of a room.
The recorder is essentially a regular client, therefore they communicate through the same
protocol.
However, the recorder's behavior is distinctly different than a regular client, as to which
they have some sharing attributes, but e.G. require different information, those differences are highlighted
in this document.

## Joining the room

### JoinSuccess

When joining a room with a timer running, the `join_success` message contains the recording status of the room.

#### Fields

The module data has the following structure:

| Field         | Type   | Required | Description                                         |
| ------------- | ------ | -------- | --------------------------------------------------- |
| `client_type` | `enum` | yes      | Either `participant` or `recorder`                  |
| `targets`     | `map`  | yes      | The map fields are different based on `client_type` |

When the `client_type` is set to `recorder`, the `targets` map contains objects of the following structure:

| Field         | Type     | Required                        | Description                                                                                                                                                             |
| ------------- | -------- | ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `stream_kind` | `enum`   | yes                             | Either `recording` or `streaming`                                                                                                                                       |
| `location`    | `string` | if `stream_kind` is `streaming` | The Url (location) to the Streaming endpoint, this location must already include the correct format to stream to. Consisting of: (`streaming_endpoint`/`streaming_key`) |

When the `client_type` is set to `participant`, the `targets` map contains objects of the following structure:

| Field        | Type     | Required                          | Description                                        |
| ------------ | -------- | --------------------------------- | -------------------------------------------------- |
| `name`       | `string` | yes                               | The name of the stream                             |
| `kind`       | `enum`   | yes                               | Either `livestream` or `recording`                 |
| `public_url` | `string` | if `kind` is `livestream`         | The Url on which the livestream can be viewed from |
| `status`     | `enum`   | if `client_type` is `participant` | The status of that specific target                 |

##### Example

For the recorder target:

```json
{
    "client_type": "recorder",
    "targets": {
        "00000000-0000-0000-0000-000000000000": {
            "stream_kind": "recording"
        },
        "00000000-0000-0000-0000-000000000001": {
            "stream_kind": "streaming",
            "location": "https://localhost/live/abc1337"
        }
    }
}
```

For the participant target:

```json
{
    "client_type": "participant",
    "targets": {
        "00000000-0000-0000-0000-000000000000": {
            "name": "Recording",
            "kind": "recording",
            "status": "active"
        },
        "00000000-0000-0000-0000-000000000001": {
            "name": "xyz321",
            "kind": "livestream",
            "public_url": "https://localhost/stream_with_me",
            "status": "error",
            "reason": {
                "code": "teapot",
                "message": "I'm a teapot"
            }
        }
    }
}
```

### Joined

When joining a room, the `joined` control event sent to all other participants contains the module-specific fields described below.

#### Fields

| Field                | Type   | Always | Description                                                  |
| -------------------- | ------ | ------ | ------------------------------------------------------------ |
| `consents_recording` | `bool` | yes    | Whether the joining participant consents to recording or not |

##### Example

```json
    ...
    "consents_recording": true
    ...

```

## Commands

### Overview

- [`start_stream`](#start)
- [`pause_stream`](#pause)
- [`stop_stream`](#stop)
- [`set_consent`](#setconsent)

### Start

The `start_stream` message can be sent by a moderator to request a stream to go live for the current room.

#### Response

A [`status`](#status) message with the streaming id is sent to every participant in the room.

#### Fields

| Field        | Type       | Required | Description                              |
| ------------ | ---------- | -------- | ---------------------------------------- |
| `action`     | `enum`     | yes      | Must be `start_stream`.                  |
| `target_ids` | `string[]` | yes      | The streaming ids that are used to start |

#### Example

```json
{
    "action": "start_stream",
    "target_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000002"],
}
```

### Pause

The `pause_stream` message can be sent by a moderator to request stream to be paused.

#### Response

A [`status`](#status) message with the streaming id is sent to every participant in the room.

#### Fields

| Field        | Type       | Required | Description                                      |
| ------------ | ---------- | -------- | ------------------------------------------------ |
| `action`     | `enum`     | yes      | Must be `pause_stream`                           |
| `target_ids` | `string[]` | yes      | The streaming ids that are supposed to be paused |

#### Example

```json
{
    "action": "pause_stream",
    "target_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000002"],
}
```

### Stop

The `stop_stream` message can be sent by a moderator to stop a stream in the current room.

#### Response

A [`status`](#status) message with the streaming id is sent to every participant in the room.

#### Fields

| Field        | Type       | Required | Description                                        |
| ------------ | ---------- | -------- | -------------------------------------------------- |
| `action`     | `enum`     | yes      | Must be `stop_stream`.                             |
| `target_ids` | `string[]` | yes      | The streaming ids that are supposed to be stopped. |

#### Example

```json
{
    "action": "stop_stream",
    "target_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001", "00000000-0000-0000-0000-000000000002"],
}
```

### SetConsent

The `SetConsent` message must be sent by every participant to consent to a recording of their video+audio.
Can always be set regardless if a recording is running or a new one is being started.
By default, the consent is always off (`false`).

#### Fields

| Field     | Type   | Required | Description                             |
| --------- | ------ | -------- | --------------------------------------- |
| `action`  | `enum` | yes      | Must be `set_consent`.                  |
| `consent` | `bool` | yes      | Set `true` if consenting to a recording |

#### Example

```json
{
    "action": "set_consent",
    "consent": true
}
```

---

## Events

### Overview

- [`status`](#status)

### Status

Is received by every participant when the status of a stream has changed.

#### Fields

| Field       | Type     | Required               | Description                                                     |
| ----------- | -------- | ---------------------- | --------------------------------------------------------------- |
| `target_id` | `string` | yes                    | The streaming id of the stream that has been updated            |
| `status`    | `enum`   | yes                    | Any of the following: `active`, `inactive`, `paused` or `error` |
| `reason`    | `Reason` | if `status` is `error` | The reason why this error has occurred.                         |

The `Reason` object has the following fields:

| Field     | Type     | Required | Description                              |
| --------- | -------- | -------- | ---------------------------------------- |
| `code`    | `string` | yes      | The error code.                          |
| `message` | `string` | yes      | Additional information about this error. |

#### Example

```json
{
    "target_id": "00000000-0000-0000-0000-000000000000",
    "status": "error",
    "reason": {
        "code": "unreachable",
        "message": "target died",
    }
}
```

---

### Error

An error has occurred while issuing a command.

#### Fields

| Field     | Type   | Required | Description                                                                         |
| --------- | ------ | -------- | ----------------------------------------------------------------------------------- |
| `message` | `enum` | yes      | Is "error".                                                                         |
| `error`   | `enum` | yes      | Is any of `insufficient_permissions`, `invalid_streaming_id` or `already_streaming` |

#### Example

```json
{
    "message": "error",
    "error": "insufficient_permissions"
}
```

---

### Recorder Error

An error has occurred while trying to start the recorder.

Error types:

- `timeout` - the recording task was not picked up by any recorder in time. This could mean that there is no recorder running or that all recorder instances are at capacity.

#### Fields

| Field     | Type   | Required | Description                 |
| --------- | ------ | -------- | --------------------------- |
| `message` | `enum` | yes      | Is `recorder_error`.        |
| `error`   | `enum` | yes      | Is currently only `timeout` |

#### Example

```json
{
    "message": "recorder_error",
    "error": "timeout"
}
```
