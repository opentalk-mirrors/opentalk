# Whiteboard

## Terminology

- **Scene** The content of the whiteboard.

## Overview

The whiteboard module allows participants to collaboratively edit an [Excalidraw](https://excalidraw.com) based whiteboard.

When breakout rooms are started while a whiteboard session is running in the main room, each breakout room gets its own session with an empty scene.
These sessions are discarded when the breakout rooms are closed.

## Joining the room

When joining a room with a running whiteboard session, the `join_success` message contains the current state of the session.
The module data has the following structure:

| Field               | Type     | Required | Description                                                              |
| ------------------- | -------- | -------- | ------------------------------------------------------------------------ |
| `scene`             | `object` | yes      | The last stored scene of the whiteboard                                  |
| `edit_restrictions` | `object` | yes      | The current edit restrictions, see [EditRestrictions](#editrestrictions) |

```json
{
  "scene": {
    "elements": []
  },
  "edit_restrictions": {
    "type": "enabled",
    "unrestricted_participants": ["00000000-0000-0000-0000-000000000001"]
  }
}
```

### EditRestrictions

| Field                       | Type       | Required                   | Description                                                          |
| --------------------------- | ---------- | -------------------------- | -------------------------------------------------------------------- |
| `type`                      | `enum`     | yes                        | Either `"disabled"` (everyone may edit) or `"enabled"`               |
| `unrestricted_participants` | `string[]` | when `type` is `"enabled"` | Ids of the non-moderator participants that are still allowed to edit |

## Commands

### Start

Allows a moderator to start a whiteboard session in the room.

#### Response

A [`Started`](#started) message is sent to every participant in the room.

If a session is already running, an [`Error`](#error) with `already_started` is returned.

#### Fields

| Field               | Type     | Required | Description                                                              |
| ------------------- | -------- | -------- | ------------------------------------------------------------------------ |
| `action`            | `enum`   | yes      | Must be `"start"`                                                        |
| `initial_scene`     | `object` | yes      | The initial scene of the whiteboard                                      |
| `edit_restrictions` | `object` | yes      | The initial edit restrictions, see [EditRestrictions](#editrestrictions) |

#### Example

```json
{
  "action": "start",
  "initial_scene": {
    "elements": []
  },
  "edit_restrictions": {
    "type": "disabled"
  }
}
```

### Stop

Allows a moderator to stop the whiteboard session in the room. The stored scene is discarded.

#### Response

A [`Stopped`](#stopped) message is sent to every participant in the room.

#### Fields

| Field    | Type   | Required | Description      |
| -------- | ------ | -------- | ---------------- |
| `action` | `enum` | yes      | Must be `"stop"` |

#### Example

```json
{
  "action": "stop"
}
```

### BroadcastVolatile

Relays arbitrary whiteboard data to all participants in the room. The data can not modify the scene, e.g. the pointer position of the sender.

Every participant is always allowed to send this command, regardless of the edit restrictions.

#### Response

A [`VolatileBroadcast`](#volatilebroadcast) message is sent to every participant in the room, except the sender.

#### Fields

| Field    | Type     | Required | Description                    |
| -------- | -------- | -------- | ------------------------------ |
| `action` | `enum`   | yes      | Must be `"broadcast_volatile"` |
| `data`   | `object` | yes      | Arbitrary whiteboard data      |

#### Example

```json
{
  "action": "broadcast_volatile",
  "data": {
    "pointer": { "x": 100, "y": 200 }
  }
}
```

### Broadcast { #broadcast-command }

Relays arbitrary whiteboard data to all participants in the room. The data modifies the scene, but is not persisted on the server. Use [`store_scene`](#storescene) to persist the scene.

Only participants that are allowed to edit the whiteboard may send this command, otherwise an [`Error`](#error) with `insufficient_permissions` is returned.

#### Response

<!-- MD051 markdownlint does not understand the `{ #id }` syntax mkdocs uses -->
<!-- markdownlint-disable-next-line MD051 -->
A [`Broadcast`](#broadcast-event) message is sent to every participant in the room, except the sender.

#### Fields

| Field    | Type     | Required | Description               |
| -------- | -------- | -------- | ------------------------- |
| `action` | `enum`   | yes      | Must be `"broadcast"`     |
| `data`   | `object` | yes      | Arbitrary whiteboard data |

#### Example

```json
{
  "action": "broadcast",
  "data": {
    "elements": []
  }
}
```

### StoreScene

Persists the scene on the server. Participants joining the room afterwards receive this scene in the `join_success` message.

Only participants that are allowed to edit the whiteboard may send this command, otherwise an [`Error`](#error) with `insufficient_permissions` is returned.

#### Response

A [`SceneStored`](#scenestored) message is sent to every participant in the room, including the sender.

#### Fields

| Field    | Type     | Required | Description             |
| -------- | -------- | -------- | ----------------------- |
| `action` | `enum`   | yes      | Must be `"store_scene"` |
| `scene`  | `object` | yes      | The scene to be stored  |

#### Example

```json
{
  "action": "store_scene",
  "scene": {
    "elements": []
  }
}
```

### Follow

Starts following another participant, i.e. the whiteboard view of the sender is synced to the view of the followed participant.

If the participant is not connected to the room, an [`Error`](#error) with `unknown_participant` is returned.

#### Response

The followed participant receives a [`FollowerGained`](#followergained) message, the sender receives a [`Followed`](#followed) message.

#### Fields

| Field            | Type     | Required | Description                         |
| ---------------- | -------- | -------- | ----------------------------------- |
| `action`         | `enum`   | yes      | Must be `"follow"`                  |
| `participant_id` | `string` | yes      | The id of the participant to follow |

#### Example

```json
{
  "action": "follow",
  "participant_id": "00000000-0000-0000-0000-000000000001"
}
```

### Unfollow

Stops following another participant.

If the participant is not connected to the room, an [`Error`](#error) with `unknown_participant` is returned.
The server does not track who follows whom, so the messages are sent even if the sender was not following the participant.

#### Response

The unfollowed participant receives a [`FollowerLost`](#followerlost) message, the sender receives an [`Unfollowed`](#unfollowed) message.

#### Fields

| Field            | Type     | Required | Description                           |
| ---------------- | -------- | -------- | ------------------------------------- |
| `action`         | `enum`   | yes      | Must be `"unfollow"`                  |
| `participant_id` | `string` | yes      | The id of the participant to unfollow |

#### Example

```json
{
  "action": "unfollow",
  "participant_id": "00000000-0000-0000-0000-000000000001"
}
```

### EnableEditRestrictions

Allows a moderator to restrict editing the whiteboard to moderators and the given participants.
Replaces any previously set list of unrestricted participants.

#### Response

An [`EditRestrictionsEnabled`](#editrestrictionsenabled) message is sent to every participant in the room.

#### Fields

| Field                       | Type       | Required | Description                                                    |
| --------------------------- | ---------- | -------- | -------------------------------------------------------------- |
| `action`                    | `enum`     | yes      | Must be `"enable_edit_restrictions"`                           |
| `unrestricted_participants` | `string[]` | yes      | Ids of the non-moderator participants that are allowed to edit |

#### Example

```json
{
  "action": "enable_edit_restrictions",
  "unrestricted_participants": ["00000000-0000-0000-0000-000000000001"]
}
```

### DisableEditRestrictions

Allows a moderator to lift the edit restrictions, so that all participants are allowed to edit the whiteboard.

#### Response

An [`EditRestrictionsDisabled`](#editrestrictionsdisabled) message is sent to every participant in the room.

#### Fields

| Field    | Type   | Required | Description                           |
| -------- | ------ | -------- | ------------------------------------- |
| `action` | `enum` | yes      | Must be `"disable_edit_restrictions"` |

#### Example

```json
{
  "action": "disable_edit_restrictions"
}
```

## Events

### Started

Is received by every participant when a moderator started a whiteboard session.

#### Fields

| Field               | Type     | Required | Description                                                              |
| ------------------- | -------- | -------- | ------------------------------------------------------------------------ |
| `message`           | `enum`   | yes      | Is `"started"`                                                           |
| `initial_scene`     | `object` | yes      | The initial scene of the whiteboard                                      |
| `edit_restrictions` | `object` | yes      | The current edit restrictions, see [EditRestrictions](#editrestrictions) |

#### Example

```json
{
  "message": "started",
  "initial_scene": {
    "elements": []
  },
  "edit_restrictions": {
    "type": "disabled"
  }
}
```

### Stopped

Is received by every participant when a moderator stopped the whiteboard session.

#### Fields

| Field     | Type   | Required | Description    |
| --------- | ------ | -------- | -------------- |
| `message` | `enum` | yes      | Is `"stopped"` |

#### Example

```json
{
  "message": "stopped"
}
```

### VolatileBroadcast

Is received by every participant when a participant sent a [`broadcast_volatile`](#broadcastvolatile) command.

Volatile broadcasts must not modify the scene. Clients must not accept volatile broadcasts that modify the scene.

#### Fields

| Field     | Type     | Required | Description                                  |
| --------- | -------- | -------- | -------------------------------------------- |
| `message` | `enum`   | yes      | Is `"volatile_broadcast"`                    |
| `sender`  | `string` | yes      | The id of the participant that sent the data |
| `data`    | `object` | yes      | Arbitrary data                               |

#### Example

```json
{
  "message": "volatile_broadcast",
  "sender": "00000000-0000-0000-0000-000000000001",
  "data": {
    "pointer": { "x": 100, "y": 200 }
  }
}
```

### Broadcast { #broadcast-event }

<!-- MD051 markdownlint does not understand the `{ #id }` syntax mkdocs uses -->
<!-- markdownlint-disable-next-line MD051 -->
Is received by every participant when a participant sent a [`broadcast`](#broadcast-command) command.

Non-volatile broadcasts are allowed to modify the scene.

#### Fields

| Field     | Type     | Required | Description                                  |
| --------- | -------- | -------- | -------------------------------------------- |
| `message` | `enum`   | yes      | Is `"broadcast"`                             |
| `sender`  | `string` | yes      | The id of the participant that sent the data |
| `data`    | `object` | yes      | Arbitrary data                               |

#### Example

```json
{
  "message": "broadcast",
  "sender": "00000000-0000-0000-0000-000000000001",
  "data": {
    "elements": []
  }
}
```

### SceneStored

Is received by every participant when a participant persisted the scene with the [`store_scene`](#storescene) command.
This event contains the full scene as stored on the server and should be used to update the local scene of the clients,
so they can sync their local scene in case it drifted apart.

#### Fields

| Field     | Type     | Required | Description         |
| --------- | -------- | -------- | ------------------- |
| `message` | `enum`   | yes      | Is `"scene_stored"` |
| `scene`   | `object` | yes      | The stored scene    |

#### Example

```json
{
  "message": "scene_stored",
  "scene": {
    "elements": []
  }
}
```

### FollowerGained

Is received by a participant when another participant started following them.

#### Fields

| Field            | Type     | Required | Description                                      |
| ---------------- | -------- | -------- | ------------------------------------------------ |
| `message`        | `enum`   | yes      | Is `"follower_gained"`                           |
| `participant_id` | `string` | yes      | The id of the participant that started following |

#### Example

```json
{
  "message": "follower_gained",
  "participant_id": "00000000-0000-0000-0000-000000000002"
}
```

### Followed

Is received by the sender of a [`follow`](#follow) command when following the participant was successful.

#### Fields

| Field            | Type     | Required | Description                              |
| ---------------- | -------- | -------- | ---------------------------------------- |
| `message`        | `enum`   | yes      | Is `"followed"`                          |
| `participant_id` | `string` | yes      | The id of the participant being followed |

#### Example

```json
{
  "message": "followed",
  "participant_id": "00000000-0000-0000-0000-000000000001"
}
```

### FollowerLost

Is received by a participant when another participant stopped following them.

#### Fields

| Field            | Type     | Required | Description                                      |
| ---------------- | -------- | -------- | ------------------------------------------------ |
| `message`        | `enum`   | yes      | Is `"follower_lost"`                             |
| `participant_id` | `string` | yes      | The id of the participant that stopped following |

#### Example

```json
{
  "message": "follower_lost",
  "participant_id": "00000000-0000-0000-0000-000000000002"
}
```

### Unfollowed

Is received by the sender of an [`unfollow`](#unfollow) command.

#### Fields

| Field            | Type     | Required | Description                                |
| ---------------- | -------- | -------- | ------------------------------------------ |
| `message`        | `enum`   | yes      | Is `"unfollowed"`                          |
| `participant_id` | `string` | yes      | The id of the participant being unfollowed |

#### Example

```json
{
  "message": "unfollowed",
  "participant_id": "00000000-0000-0000-0000-000000000001"
}
```

### EditRestrictionsEnabled

Is received by every participant when a moderator enabled the edit restrictions.

#### Fields

| Field                       | Type       | Required | Description                                                          |
| --------------------------- | ---------- | -------- | -------------------------------------------------------------------- |
| `message`                   | `enum`     | yes      | Is `"edit_restrictions_enabled"`                                     |
| `unrestricted_participants` | `string[]` | yes      | Ids of the non-moderator participants that are still allowed to edit |

#### Example

```json
{
  "message": "edit_restrictions_enabled",
  "unrestricted_participants": ["00000000-0000-0000-0000-000000000001"]
}
```

### EditRestrictionsDisabled

Is received by every participant when a moderator disabled the edit restrictions.

#### Fields

| Field     | Type   | Required | Description                       |
| --------- | ------ | -------- | --------------------------------- |
| `message` | `enum` | yes      | Is `"edit_restrictions_disabled"` |

#### Example

```json
{
  "message": "edit_restrictions_disabled"
}
```

### Error

Received when something went wrong processing messages sent to the server.

#### Fields

| Field     | Type   | Always | Description                                       |
| --------- | ------ | ------ | ------------------------------------------------- |
| `message` | `enum` | yes    | Is `"error"`                                      |
| `error`   | `enum` | yes    | Exhaustive list of error strings, see table below |

| Error                      | Description                                                        |
| -------------------------- | ------------------------------------------------------------------ |
| `insufficient_permissions` | The requesting user has insufficient permissions for the operation |
| `already_started`          | A whiteboard session is already running in the room                |
| `not_started`              | No whiteboard session is running in the room                       |
| `unknown_participant`      | The target participant is not connected to the room                |

#### Example

```json
{
  "message": "error",
  "error": "insufficient_permissions"
}
```
