# Automoderation

The Automoderation feature allows one to let the server organize a discussion (e.g. manage queue of speakers, select new speakers, etc.)

## Kinds of operation

The automoderation feature uses a state machine which, depending on the configuration drive the state of the conference.
The core concept is a selection strategy:

There are 4 strategies:

- `none`
- `playlist`
- `random`
- `nomination`

Additionally there is a set of config option that need to be set when the automod feature is started.

|                          |                                                                                                                          |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| `show_list`              | When true, the module will relay information about the last and upcoming speakers to the frontend.                       |
| `consider_hand_raise`    | When true, the module will add the participant raising their hand to the playlist.                                       |
| `time_limit`             | Time limit each speaker has before its speaking status get revoked                                                       |
| `allow_double_selection` | Depending on the `selection_strategy` this will prevent participants to become speaker twice in a single automod session |
| `animation_on_random`    | When true, the frontend will play an animation when a random selection is being made.                                    |
| `allow_list`             | Depending on the selection strategy, the list of Participant that can be chosen from.                                    |
| `playlist`               | Ordered list of queued participants.                                                                                     |

The selection of the first speaker must be done by the frontend then, depending of the `selection_strategy`, the automod module will continue running until finished or stopped.

Once the active speaker yields or their time runs out, their automod module is responsible to select the next speaker (if the `selection_strategy` requires it).
This behavior **MUST** only be executed after ensuring that this participant is in fact still the speaker.

If the participant leaves while being speaker, its automod-module must execute the same behavior as if the participants simply yielded without selecting the next one (which would be required for the `"nominate"` `selection_strategy`.
A moderator has to intervene in this situation).

Moderators will always be able to execute a re-selection of the current speaker regardless of the `selection_strategy`.

### Selection Strategy: None

No automatic reselection happens after the current speaker yields.
The next one must always be selected by the moderator.
The moderator may choose a participant directly or let the controller choose one randomly.
For that the controller holds a `allow_list` which is a set of participants which are able to be randomly selected.
Furthermore the controller will hold a list of start/stop speaker events.
That list can be used to avoid double selections (option) when randomly choosing a participant.

In this selection strategy the following holds.

- `allow_list` is used to describe the pool of valid manual selection targets.
- `playlist` is not used.

### Selection Strategy: Playlist

The playlist-strategy requires a playlist of participants. This list will be stored ordered inside the controller.
Whenever a speaker yields the controller will automatically choose the next participant in the list to be the next speaker.

A moderator may choose to skip over a speaker.
That can be done by selecting the next one or let the controller choose someone random from the playlist.
The playlist can, while the automod is active, be edited.

In this selection strategy the following holds.

- `allow_list` is not used.
- `playlist` is a ordered list of participants which will get used to select the next participant when yielding.

It is also used as a pool to select participants randomly from (moderator command `select`).

### Selection Strategy: Random

This strategy behaves like `None` but will always choose the next speaker randomly from the `allow_list` as soon as the current speaker yields.

In this selection strategy the following holds.

- `allow_list` is used to describe the pool of valid selection targets.
- `playlist` is not used.

### Selection Strategy: Nomination

This strategy behaves like `None` but requires the current speaker to nominate the next participant to be speaker.
The nominated participant **MUST** be inside the `allow_list` and if double selection is not enabled the controller will check if the nominated participant already was a speaker.

In this selection strategy the following holds.

- `allow_list` is used to describe the pool of valid manual selection targets.
- `playlist` is not used.

## Joining the room

### JoinSuccess

When joining a room, the `join_success` control event contains the module-specific fields decribed below.

#### Fields

| Field     | Type           | Always | Description                          |
| --------- | -------------- | ------ | ------------------------------------ |
| `config`  | `PublicConfig` | yes    | Configuration of the auto-moderation |
| `speaker` | `string`       | no     | The currently active speaker         |

**`PublicConfig` fields**:

| Field                    | Type       | Required | Description                           |
| ------------------------ | ---------- | -------- | ------------------------------------- |
| `selection_strategy`     | `enum`     | yes      | See the same field in [Start](#start) |
| `issued_by`              | `string`   | yes      | See the same field in [Start](#start) |
| `show_list`              | `bool`     | yes      | See the same field in [Start](#start) |
| `consider_hand_raise`    | `bool`     | yes      | See the same field in [Start](#start) |
| `time_limit`             | `int`      | no       | See the same field in [Start](#start) |
| `allow_double_selection` | `bool`     | yes      | See the same field in [Start](#start) |
| `animation_on_random`    | `bool`     | yes      | See the same field in [Start](#start) |
| `auto_append_on_join`    | `bool`     | yes      | See the same field in [Start](#start) |
| `history`                | `string[]` | no       | See the same field in [Start](#start) |
| `remaining`              | `string[]` | no       | See the same field in [Start](#start) |

##### Example

```json
{
    "config": {
        "selection_strategy": "none",
        "show_list": true,
        "consider_hand_raise": false,
        "time_limit": 10000,
        "allow_double_selection": false,
        "animation_on_random": true,
        "auto_append_on_join": false,
        "allow_list": ["00000000-0000-0000-0000-000000000000"],
        "playlist": ["00000000-0000-0000-0000-000000000000"]
    },
    "speaker": "00000000-0000-0000-0000-000000000000"
}
```

### Joined

When joining a room, the `joined` control event sent to all other participants contains the module-specific fields decribed below.

## Commands

Commands are issued by a participant to start or interact with a automod session.

### Overview

- [`start`](#start)
- [`edit`](#edit)
- [`stop`](#stop)
- [`select`](#select)
- [`yield`](#yield)

### Start

The `Start` message can be send by a moderator (or user with the correct permissions) to start a automod session.

#### Fields

| Field                    | Type       | Required | Validation                                                         | Description                                                                                                          |
| ------------------------ | ---------- | -------- | ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------- |
| `action`                 | `enum`     | yes      | Must be `"start"`                                                  |                                                                                                                      |
| `selection_strategy`     | `enum`     | yes      | Must be one of  `"none"`, `"playlist"`, `"random"`, `"nomination"` | The used selection strategy                                                                                          |
| `show_list`              | `bool`     | yes      |                                                                    | When true, the module will relay information about the last and upcoming speakers to the frontend.                   |
| `consider_hand_raise`    | `bool`     | yes      |                                                                    | When true, the module will add the participant raising their hand to the playlist                                    |
| `time_limit`             | `int`      | no       |                                                                    | Time limit in milliseconds speaker has before its speaking status get revoked                                        |
| `allow_double_selection` | `bool`     | yes      |                                                                    | Depending on `selection_strategy` this will prevent participants to become speaker twice in a single automod session |
| `animation_on_random`    | `bool`     | yes      |                                                                    | When true, the frontend will play an animation when a random selection is being made                                 |
| `auto_append_on_join`    | `bool`     | yes      |                                                                    | When true, joining participants are automatically added to the speaker list                                          |
| `allow_list`             | `string[]` | no       | Valid `ParticipantIds`                                             | Depending on `selection_strategy`, the list of Participant that can be chosen from.                                  |
| `playlist`               | `string[]` | no       | Valid `ParticipantIds`                                             | Ordered list of queued participants                                                                                  |

##### Example

```json
{
    "action": "start",
    "selection_strategy": "none",
    "show_list": true,
    "consider_hand_raise": false,
    "time_limit": 10000,
    "allow_double_selection": false,
    "animation_on_random": true,
    "auto_append_on_join": false,
    "allow_list": ["00000000-0000-0000-0000-000000000000"],
    "playlist": ["00000000-0000-0000-0000-000000000000"]

}
```

#### Response

A [Started](#started) message is sent to all participants that are currently in the room.

---

### Stop

Stop the currently active automoderation session.

#### Fields

| Field    | Type   | Required | Validation        |
| -------- | ------ | -------- | ----------------- |
| `action` | `enum` | yes      | Must be `"stop"`. |

##### Example

```json
{
    "action": "stop",
}
```

#### Response

A [Stopped](#stopped) message with the `stopped_by_moderator` reason is sent to all participants that are currently in the room.

---

### Edit

Set either the `allow_list` or `playlist` or both.

#### Fields

| Field        | Type       | Required | Validation            | Description       |
| ------------ | ---------- | -------- | --------------------- | ----------------- |
| `action`     | `enum`     | yes      |                       | Must be `"edit"`. |
| `allow_list` | `string[]` | no       | Valid `ParticipantId` |                   |
| `playlist`   | `string[]` | no       | Valid `ParticipantId` |                   |

##### Example

Setting the allowlist to contain the zero participantId and the playlist to be empty

```json
{
    "action": "edit",
    "allow_list": ["00000000-0000-0000-0000-000000000000"],
    "playlist": []

}
```

Setting the allowlist to contain the zero and one participantId. The playlist is not changed

```json
{
    "action": "edit",
    "allow_list": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
}
```

#### Response

A [Remaining Updated](#remaining-updated) message is sent to all participants that are currently in the room.

---

### Select

Select a user to be active speaker.

#### Fields

| Field               | Type     | Required                 | Validation                                                   | Description                                                                             |
| ------------------- | -------- | ------------------------ | ------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| `action`            | `enum`   | yes                      | Must be `"select,`.                                          |                                                                                         |
| `how`               | `enum`   | yes                      | Must be one of `"none"`, `"random,`, `"next"`, `"specific"`. |                                                                                         |
| `participant`       | `string` | when `how` == `specific` | Valid `ParticipantId`                                        |                                                                                         |
| `keep_in_remaining` | `bool`   | when `how` == `specific` |                                                              | If true the selected participant will not be removed from either the allow- or playlist |

##### Example

Select a specific speaker

```json
{
    "action": "select",
    "how": "random"
}
```

Select a specific speaker

```json
{
    "action": "select",
    "how": "specific",
    "participant": "00000000-0000-0000-0000-000000000000",
    "keep_in_remaining": true

}
```

#### Response

Depending on the configuration a [Speaker Updated](#speaker-updated) or a [Start Animation](#start-animation) event is sent in response to this action. When a `Start Animation` event is sent, a `Speaker Update` follows.

---

### Yield

User yields their speaker status.

#### Fields

| Field    | Type     | Required | Validation            | Description                                                                                                               |
| -------- | -------- | -------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `action` | `enum`   | yes      | Must be `"yield"`.    |                                                                                                                           |
| `next`   | `string` | no       | Valid `ParticipantId` | In some cases a user must select the next participant to be speaker. This is the case when the strategy is `"nomination"` |

##### Example

```json
{
    "action": "yield",
    "next": "00000000-0000-0000-0000-000000000000",
}
```

#### Response

Depending on the configuration a [Speaker Updated](#speaker-updated) or a [Start Animation](#start-animation) event is sent in response to this action. When a `Start Animation` event is sent, a `Speaker Update` follows.

---

## Events

Events are received by participants when the automod session state is changed.

### Started

Signals the start of an automod session.

#### Fields

| Field                    | Type       | Required | Validation                                                         | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| ------------------------ | ---------- | -------- | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `message`                | `enum`     | yes      | Must be `"started"`                                                |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `selection_strategy`     | `enum`     | yes      | Must be one of  `"none"`, `"playlist"`, `"random"`, `"nomination"` | The used selection strategy                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `issued_by`              | `string`   | yes      | Valid `ParticipantId`                                              | The ID of the participant who started the automoderation session                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `show_list`              | `bool`     | yes      |                                                                    | When true, the module will relay information about the last and upcoming speakers to the frontend                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `consider_hand_raise`    | `bool`     | yes      |                                                                    | When true, the module will add the participant raising their hand to the playlist                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `time_limit`             | `int`      | no       |                                                                    | Time limit in milliseconds speaker has before its speaking status get revoked                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `allow_double_selection` | `bool`     | yes      |                                                                    | Depending on `selection_strategy` this will prevent participants to become speaker twice in a single automod session                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `animation_on_random`    | `bool`     | yes      |                                                                    | When true, the frontend will play an animation when a random selection is being made                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `auto_append_on_join`    | `bool`     | yes      |                                                                    | When true, joining participants are automatically added to the speaker list                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `history`                | `string[]` | no       | Valid `ParticipantId`s                                             | Optional modification of the history. If set the frontend MUST replace its history with the given one. If not set the frontend MUST keep its current history.                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `remaining`              | `string[]` | no       | Valid `ParticipantId`s                                             | Optional modification of the remaining participants. Remaining participants must be interpreted differently depending on the selection strategy.  E.g. in the playlist strategy, `remaining` lists the participants left inside the playlist. All other strategies will use `remaining` (if at all) to list all participants (if public) that are eligible to be selected. This will only be set if `selection_strategy` is set to `"playlist"`. If set, the frontend MUST replace its list with the given one. If not set, the frontend MUST keep its current remaining list |

##### Example

```json
{
    "message": "started",
    "selection_strategy": "none",
    "issued_by": "00000000-0000-0000-0000-000000000000",
    "show_list": true,
    "consider_hand_raise": false,
    "time_limit": 5000,
    "allow_double_selection": false,
    "animation_on_random": true,
    "auto_append_on_join": false,
    "history": [
        "00000000-0000-0000-0000-000000000000"
    ],
    "remaining": [
        "00000000-0000-0000-0000-000000000000"
    ]

}
```

---

### Stopped

Signals the end of an automod session.

#### Fields

| Field       | Type     | Required | Description                                                                                                                        |
| ----------- | -------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `message`   | `enum`   | yes      | Is `"stopped"`.                                                                                                                    |
| `reason`    | `enum`   | yes      | `"stopped_by_moderator"` or `"session_finished"`.                                                                                  |
| `issued_by` | `string` | no       | The participant ID of the moderator who issues the [Stopped](#stopped) command and it's only present for `"stopped_by_moderator"`. |

##### Examples

Stopped by a moderator:

```json
{
    "message": "stopped",
    "reason": "stopped_by_moderator",
    "issued_by": "00000000-0000-0000-0000-000000000000"
}
```

Session finished:

```json
{
    "message": "stopped",
    "reason": "session_finished"
}
```

---

### Speaker Updated

The current speaker has been updated.

This event will ALWAYS notify of a speaker change, even if the speaker is the same participant as before, it MUST be handled as changed.

Both `history` and `remaining`: If the field is set it contains the complete new list. If it doesn't exist it must be treated as unchanged.

#### Fields

| Field       | Type       | Required | Validation              | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ----------- | ---------- | -------- | ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `message`   | `enum`     | yes      | Is `"speaker_updated"`. |                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `speaker`   | `string`   | no       | Valid`"ParticipantId"`. | Optional currently active Speaker. If not set, no speaker is currently active. Can be the same participant as before                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `history`   | `string[]` | no       | Valid `ParticipantId`s  | Optional modification of the history. If set, the frontend MUST replace its history with the given one. If not set the frontend MUST keep its current history.                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `remaining` | `string[]` | no       | Valid `ParticipantId`s  | Optional modification of the remaining participants. Remaining participants must be interpreted differently depending on the selection strategy.  E.g. in the playlist moderation remaining lists the participants left inside the playlist. All other strategies will use `remaining` (if at all) to list all participants (if public) that are eligible to be selected. This will only be set when using the `"playlist"` `selection_strategy`. If set the frontend MUST replace its remaining list with the given one. If not set the frontend MUST keep its current remaining list. |

##### Example

```json
{
    "message": "speaker_updated",
    "speaker": "00000000-0000-0000-0000-000000000000",
    "history": [],
    "remaining": [
        "00000000-0000-0000-0000-000000000000"
    ]

}
```

---

### Remaining Updated

The remaining list has been updated

A modification of the remaining list has taken place, because someone edited the list by hand or it got modified because a participant left/joined.

#### Fields

| Field       | Type       | Required | Validation                | Description |
| ----------- | ---------- | -------- | ------------------------- | ----------- |
| `message`   | `enum`     | yes      | Is `"remaining_updated"`. |             |
| `remaining` | `string[]` | yes      | Valid `ParticipantId`s    |             |

##### Example

Stopped with valid results:

```json
{
    "message": "remaining_updated",
    "remaining": [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002"
    ]

}
```

---

### Start Animation

Tell the frontend to start the animation for random selection.
The animation must yield the result specified by this message.

#### Fields

| Field     | Type       | Required | Validation              | Description |
| --------- | ---------- | -------- | ----------------------- | ----------- |
| `message` | `enum`     | yes      | Is `"start_animation"`. |             |
| `pool`    | `string[]` | yes      | Valid `ParticipantId`s  |             |
| `result`  | `string`   | yes      | Valid `ParticipantId`   |             |

##### Example

```json
{
    "message": "start_animation",
    "pool": [],
    "result": "00000000-0000-0000-0000-000000000000"
}
```

---

### Error

The error event is a message that may be triggered by syntactically correct but invalid commands inside the `automod` namespace
and therefore could be considered a kind of response. Errors must be handled outside of any context as they are considered events
that can happen at any time. (e.g. an `internal` error may occur at any time to signal an internal problem)

#### Fields

| Field     | Type   | Required | Description                                       |
| --------- | ------ | -------- | ------------------------------------------------- |
| `message` | `enum` | yes      | Is `"error"`.                                     |
| `error`   | `enum` | yes      | Exhaustive list of error strings, see table below |

| Error                      | Description                                                                                                        |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `invalid_selection`        | The selection made by the frontend was invalid. Can originate from the `"start"`, `"yield"` or `"select"` command. |
| `insufficient_permissions` | The issued command can only be issued by a moderator, but the issuer isn't one.                                    |
| `session_already_running`  | Attempted to start a new session when another active session is already running.                                   |

#### Example

```json
{
    "message": "error",
    "error": "invalid_selection"
}
```
