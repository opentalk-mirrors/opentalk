# Timer

## Joining the room

### JoinSuccess

When joining a room with a timer running, the `join_success` control event contains the module-specific fields described below.

#### Fields

| Field                 | Type     | Always                     | Description                                                          |
| --------------------- | -------- | -------------------------- | -------------------------------------------------------------------- |
| `timer_id`            | `string` | yes                        | The timer id (uuid)                                                  |
| `started_at`          | `string` | yes                        | RFC 3339 timestamp of when the timer started                         |
| `kind`                | `enum`   | yes                        | Either `"countdown"` or `"stopwatch"`                                |
| `ends at`             | `string` | if `kind` is `"countdown"` | optional RFC 3339 timestamp of when the timer will end               |
| `style`               | `string` | no                         | An optional style tag to identify a timer across frontend clients    |
| `title`               | `string` | no                         | An optional title for the timer                                      |
| `ready_check_enabled` | `bool`   | yes                        | Enables/Disables participants to send a `update_ready_status` action |
| `ready_status`        | `bool`   | no                         | The ready status of the participant for a currently active timer     |

##### Example

Countdown:

```json
{
    "message": "started",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "kind": "countdown",
    "started_at": "1970-01-01T00:00:00Z",
    "ends_at": "1970-01-01T00:00:00Z",
    "title": "Testing the timer!",
    "ready_check_enabled": false,
    "ready_status": false
}
```

### Joined

When joining a room with a timer running, the `joined` control event sent to all other participants contains the module-specific fields described below.

#### Fields

| Field          | Type   | Always | Description                                                      |
| -------------- | ------ | ------ | ---------------------------------------------------------------- |
| `ready_status` | `bool` | no     | The ready status of the participant for a currently active timer |

##### Example

```json
    ...
    "ready_status": true,
    ...

```

---

## Overview

The timer module allows a moderator to start a timer for all participants in the room.

There are two variations of the timer: `"countdown"` and `"stopwatch"`.

When the `"countdown"` kind is configured, the timer continues to run until its duration expires or if a moderator stops it beforehand.

When the `"stopwatch"` kind is configured, the timer continues to run until a moderator stops it.

There can only be one timer present at a time. Due to a current limitation of our module system,
all participants will receive a `stopped` event when the creator of a timer leaves the room. This behavior shall be fixed in the future.

## Commands

### Start

The `Start` action can be sent by a moderator to start a new timer.

Can return [Error](#error) of kind `insufficient_permissions`, `invalid_duration` or `timer_already_running`

#### Fields

| Field                | Type     | Required                   | Description                                                         |
| -------------------- | -------- | -------------------------- | ------------------------------------------------------------------- |
| `action`             | `enum`   | yes                        | Must be `"start"`                                                   |
| `kind`               | `enum`   | yes                        | Either `"countdown"` or `"stopwatch"`                               |
| `duration`           | `int`    | if `kind` is `"countdown"` | The duration (seconds) of the timer                                 |
| `title`              | `string` | no                         | The optional title for the timer                                    |
| `style`              | `string` | no                         | An optional style tag to identify a timer across frontend clients   |
| `enable_ready_check` | `bool`   | yes                        | Enables/Disables participants to send a `update_ready_check` action |

When the `kind` is set to `"countdown"`, the module will automatically send a [Stopped](#stopped) message when the `duration` expires.

##### Examples

Countdown:

```json
{
    "action": "start",
    "kind": "countdown",
    "duration": 20,
    "title": "Testing the timer!",
    "enable_ready_check": false
}
```

Countdown with style:

```json
{
    "action": "start",
    "kind": "countdown",
    "duration": 180,
    "style": "coffee_break",
    "enable_ready_check": true
}
```

Stopwatch:

```json
{
    "action": "start",
    "kind": "stopwatch",
    "title": "Testing the stopwatch!",
    "enable_ready_check": false
}
```

#### Response

Each participant receives a [Started](#started) message with the configuration of the timer.

---

### Stop

The `Stop` action can be sent by a moderator to stop a timer.

#### Fields

| Field      | Type     | Required | Description                                           |
| ---------- | -------- | -------- | ----------------------------------------------------- |
| `action`   | `enum`   | yes      | Must be `"stop"`                                      |
| `timer_id` | `string` | yes      | The timer id (uuid)                                   |
| `reason`   | `string` | no       | An optional reason for the stop, set by the moderator |

##### Examples

```json
        {
            "action": "stop",
            "timer_id": "00000000-0000-0000-0000-000000000000",
            "reason": "test"
        }
```

#### Response

Each participant receives a [Stopped](#stopped) message.

---

### Update Ready Status

Set the ready status of a participant.

Can be sent by any participant only when `ready_check` is set to true in [Start](#start).

#### Fields

| Field      | Type     | Required | Description                         |
| ---------- | -------- | -------- | ----------------------------------- |
| `action`   | `enum`   | yes      | Must be `"update_ready_status"`     |
| `timer_id` | `string` | yes      | The timer id (uuid)                 |
| `status`   | `bool`   | yes      | The ready status of the participant |

##### Examples

```json
{
    "action": "update_ready_status",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "status": true
}
```

#### Response

Each participant receives a [Control Update](#updated-ready-status) event with the ready status of the participant that was updated.

---

## Events

### Started

A timer has been started.

#### Fields

| Field                 | Type     | Always                     | Description                                                          |
| --------------------- | -------- | -------------------------- | -------------------------------------------------------------------- |
| `message`             | `enum`   | yes                        | Is `"started"`                                                       |
| `timer_id`            | `string` | yes                        | The timer id (uuid)                                                  |
| `kind`                | `enum`   | yes                        | Either `"countdown"` or `"stopwatch"`                                |
| `started_at`          | `string` | yes                        | RFC 3339 timestamp of when the timer started                         |
| `ends at`             | `string` | if `kind` is `"countdown"` | optional RFC 3339 timestamp of when the timer will end               |
| `title`               | `string` | no                         | An optional title for the timer                                      |
| `style`               | `string` | no                         | An optional style tag to identify a timer across frontend clients    |
| `ready_check_enabled` | `bool`   | yes                        | Enables/Disables participants to send a `update_ready_status` action |

##### Example

Countdown:

```json
{
    "message": "started",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "kind": "countdown",
    "started_at": "1970-01-01T00:00:00Z",
    "ends_at": "1970-01-01T00:00:00Z",
    "title": "Testing the timer!",
    "ready_check_enabled": false
}
```

Countdown with style:

```json
{
    "message": "started",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "kind": "countdown",
    "started_at": "1970-01-01T00:00:00Z",
    "ends_at": "1970-01-01T00:00:00Z",
    "style": "coffee_break",
    "ready_check_enabled": false
}
```

Stopwatch:

```json
{
    "message": "started",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "kind": "stopwatch",
    "started_at": "1970-01-01T00:00:00Z",
    "ends_at": "1970-01-01T00:00:00Z",
    "title": "Testing the stopwatch!",
    "ready_check_enabled": false
}
```

---

### Stopped

A timer has been stopped.

#### Fields

| Field            | Type     | Always                        | Description                                                               |
| ---------------- | -------- | ----------------------------- | ------------------------------------------------------------------------- |
| `message`        | `enum`   | yes                           | Is `"stopped"`                                                            |
| `timer_id`       | `string` | yes                           | The timer id (uuid)                                                       |
| `kind`           | `enum`   | yes                           | The kind of stop. Must be `"by_moderator"`, `"expired"`, `"creator_left"` |
| `participant_id` | `string` | when `kind` is `by_moderator` | The participant id of the moderator that stopped the timer                |
| `reason`         | `string` | when `kind` is `by_moderator` | Optional reason. Set by the moderator                                     |

__StopKind:__

| Kind           | Description                                   |
| -------------- | --------------------------------------------- |
| `by_moderator` | The timer was stopped manually by a moderator |
| `expired`      | The duration of the timer has expired         |
| `creator_left` | The creator of the timer has left the room    |

##### Example

by moderator:

```json
{
    "message": "stopped",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "kind": "by_moderator",
    "participant_id": "00000000-0000-0000-0000-000000000000",
    "reason": "A good reason!"
}
```

expired:

```json
{
    "message": "stopped",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "kind": "expired",
}
```

creator left:

```json
{
    "message": "creator_left",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "kind": "expired",
}
```

---

### Updated ready status

A participant has updated its ready status.

#### Fields

| Field            | Type     | Always | Description                                 |
| ---------------- | -------- | ------ | ------------------------------------------- |
| `message`        | `enum`   | yes    | Is `"updated_ready_status"`                 |
| `timer_id`       | `string` | yes    | The timer id (uuid)                         |
| `participant_id` | `string` | yes    | The participant that had its status updated |
| `status`         | `bool`   | yes    | The participants status                     |

##### Examples

```json
{
    "message": "updated_ready_status",
    "timer_id": "00000000-0000-0000-0000-000000000000",
    "participant_id": "00000000-0000-0000-0000-000000000000",
    "status": true,
}
```

---

### Error

An error has occurred while issuing a command.

#### Fields

| Error                      | Description                                                |
| -------------------------- | ---------------------------------------------------------- |
| `insufficient_permissions` | The issued command requires greater permissions            |
| `invalid_duration`         | The provided duration (in a start request) is invalid      |
| `timer_already_running`    | A timer is already running while trying to start a new one |

##### Examples

```json
{
    "message":"error",
    "error":"insufficient_permissions"
}
```

```json
{
    "message":"error",
    "error":"invalid_duration"
}
```

```json
{
    "message":"error",
    "error":"timer_already_running"
}
```
