# Poll

## Joining the room

### JoinSuccess

When joining a room with a poll running, the `join_success` control event contains the module-specific fields described below.

#### Fields

| Field             | Type       | Always | Description                                             |
| ----------------- | ---------- | ------ | ------------------------------------------------------- |
| `id`              | `string`   | yes    | Id of the poll                                          |
| `topic`           | `string`   | yes    | Topic of the poll                                       |
| `live`            | `bool`     | yes    | The state of the poll is reported immediately           |
| `multiple_choice` | `bool`     | yes    | Multiple choices can be selected                        |
| `choices`         | `Choice[]` | yes    | The available choices to vote on, see [Choice](#choice) |
| `started`         | `string`   | yes    | Timestamp for when the poll was started                 |
| `duration`        | `int`      | yes    | Duration of the poll in seconds                         |

##### Example

```json
{
    "id": "00000000-0000-0000-0000-000000000000",
    "topic": "Yes or No?",
    "live": true,
    "multiple_choice": true,
    "choices": [
        { "id": 0, "content": "first choice" },
        { "id": 1, "content": "second choice" },
        { "id": 2, "content": "third choice" }
    ],
    "started": "2022-10-22T11:22:33Z",
    "duration": 60000
}
```

### Joined

When joining a room, the `joined` control event sent to all other participants does not contain module-specific data.

---

## Commands

Commands are issued by a participate to start or interact with a poll.

### Start

The `Start` message can be send by a user to start a new poll. New polls can only be created if the current one has finished or has been canceled.

#### Fields

| Field             | Type       | Required | Description                                                        |
| ----------------- | ---------- | -------- | ------------------------------------------------------------------ |
| `action`          | `enum`     | yes      | Must be `"start"`                                                  |
| `topic`           | `string`   | yes      | Topic of the poll                                                  |
| `live`            | `bool`     | no       | The state of the poll is reported immediately, defaults to `false` |
| `multiple_choice` | `bool`     | no       | Multiple choices can be selected, defaults to `false`              |
| `choices`         | `string[]` | yes      | Non empty array of strings which each describe a choice            |
| `duration`        | `int`      | no       | Duration of the poll in seconds                                    |

##### Example

```json
{
    "action": "start",
    "topic": "some topic",
    "live": true,
    "multiple_choice": true,
    "choices": ["first choice", "seconds choice", "third choice"],
    "duration": 60000
}
```

#### Response

A [Started](#started) message is sent to all participants that are currently in the room.

Can return [Error](#error) of kind `insufficient_permissions`, `invalid_choice_count`, `invalid_choice_description`, `invalid_topic_length`, `invalid_duration`, and `still_running`.

---

### Vote

Cast your vote for a poll with the specified `poll_id`. Each participant can only vote once per poll.

If a vote is started with the `live` flag set to `true` a [LiveUpdate](#liveupdate) is sent to all participants.

For backwards compatibility, both an array of choice ids as well as a single choice id can be specified.
The `choice_id` single value field takes precedence over the `choice_ids` array field.

#### Fields

| Field        | Type     | Required | Description                                                                        |
| ------------ | -------- | -------- | ---------------------------------------------------------------------------------- |
| `action`     | `enum`   | yes      | Must be `"vote"`                                                                   |
| `poll_id`    | `string` | yes      | ID of the poll                                                                     |
| `choice_id`  | `int`    | no       | ID of a single choice, takes precedence over `choice_ids`                          |
| `choice_ids` | `int[]`  | no       | ID of multiple choices, empty to abstain, only evaluated if `choice_id` is missing |

To abstain, both `choice_id` and `choice_ids` can be omitted.

##### Example

```json
{
    "action": "vote",
    "poll_id": "00000000-0000-0000-0000-000000000000",
    "choice_id": 1
}
```

#### Response

Can return [Error](#error) of kind `invalid_poll_id`, `invalid_choice_id` and `multiple_choice_not_allowed`.

---

### Finish

Finish a poll prematurely.

A [Done](#done) message will be sent to all other participants.

#### Fields

| Field    | Type     | Required | Description        |
| -------- | -------- | -------- | ------------------ |
| `action` | `enum`   | yes      | Must be `"finish"` |
| `id`     | `string` | yes      | ID of the poll     |

##### Example

```json
{
    "action": "finish",
    "id": "00000000-0000-0000-0000-000000000000"
}
```

#### Response

Can return [Error](#error) of kind `insufficient_permissions` and `invalid_poll_id`.

---

## Events

Events are received by participants when the poll state has changed.

### Started

A poll has been started.

#### Fields

| Field             | Type       | Always | Description                                             |
| ----------------- | ---------- | ------ | ------------------------------------------------------- |
| `message`         | `enum`     | yes    | Is `"started"`                                          |
| `id`              | `string`   | yes    | Id of the poll                                          |
| `topic`           | `string`   | yes    | Topic of the poll                                       |
| `live`            | `bool`     | yes    | The state of the poll is reported immediately           |
| `multiple_choice` | `bool`     | yes    | Multiple choices can be selected                        |
| `choices`         | `Choice[]` | yes    | The available choices to vote on, see [Choice](#choice) |
| `duration`        | `int`      | yes    | Duration of the poll in seconds                         |

##### Example

```json
{
    "message": "started",
    "id": "00000000-0000-0000-0000-000000000000",
    "topic": "Yes or No?",
    "live": true,
    "multiple_choice": true,
    "choices": [
        { "id": 0, "content": "first choice" },
        { "id": 1, "content": "second choice" },
        { "id": 2, "content": "third choice" }
    ],
    "duration": 60000
}
```

---

### LiveUpdate

A poll has been updated.

#### Fields

| Field     | Type       | Always | Description                      |
| --------- | ---------- | ------ | -------------------------------- |
| `message` | `enum`     | yes    | Is `"live_update"`               |
| `id`      | `string`   | yes    | Id of the poll                   |
| `results` | `Choice[]` | yes    | List of choices and their scores |

__`Choice` Fields:__

| Field   | Type  | Always | Description                    |
| ------- | ----- | ------ | ------------------------------ |
| `id`    | `int` | yes    | ID of the choice               |
| `count` | `int` | yes    | Count of votes for this choice |

##### Example

```json
{
    "message": "live_update",
    "id": "00000000-0000-0000-0000-000000000000",
    "choices": [
        { "id": 0, "count": 3 },
        { "id": 1, "count": 7 },
        { "id": 2, "count": 1 }
    ]
}
```

---

### Done

A poll has been completed due to expiration or a moderator issuing the [Finish](#finish) command.

#### Fields

| Field     | Type       | Always | Description                      |
| --------- | ---------- | ------ | -------------------------------- |
| `message` | `enum`     | yes    | Is `"live_update"`               |
| `id`      | `string`   | yes    | Id of the poll                   |
| `results` | `Choice[]` | yes    | List of choices and their scores |

__`Choice` Fields:__

| Field   | Type  | Always | Description                    |
| ------- | ----- | ------ | ------------------------------ |
| `id`    | `int` | yes    | ID of the choice               |
| `count` | `int` | yes    | Count of votes for this choice |

##### Example

```json
{
    "message": "live_update",
    "id": "00000000-0000-0000-0000-000000000000",
    "choices": [
        { "id": 0, "count": 3 },
        { "id": 1, "count": 7 },
        { "id": 2, "count": 1 }
    ]
}
```

---

### Error

An error has occurred when issuing a command

| Field     | Type   | Always | Description                           |
| --------- | ------ | ------ | ------------------------------------- |
| `message` | `enum` | yes    | Is `"error"`                          |
| `error`   | `enum` | yes    | Variant of the error, see table below |

| Error                          | Description                                                                                                    |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| `insufficient_permissions`     | The issued command requires greater permissions                                                                |
| `invalid_choice_count`         | The [Start](#start) command specified an invalid amount of choices (must be greater than 2 and fewer than 64)  |
| `invalid_poll_id`              | Unknown poll id                                                                                                |
| `invalid_choice_id`            | Unknown choice id                                                                                              |
| `multiple_choices_not_allowed` | Issued when a [Vote](#vote) command contains more than one choice but multiple choice is disabled for the poll |
| `invalid_choice_description`   | Given choice description was invalid (length must be between 2 and 100 bytes)                                  |
| `invalid_topic_length`         | Given topic length was invalid (must be between 2 and 100 bytes)                                               |
| `invalid_duration`             | Invalid poll duration (must be greater than 2 seconds and shorter than 1 hour)                                 |
| `still_running`                | Tried to start a poll while a poll is still running                                                            |

## Shared Types

### Choice

The description of a choice that can be voted on.

#### Fields

| Field     | Type     | Always | Description                       |
| --------- | -------- | ------ | --------------------------------- |
| `id`      | `int`    | yes    | ID of the choice                  |
| `content` | `string` | yes    | Content/Description of the choice |
