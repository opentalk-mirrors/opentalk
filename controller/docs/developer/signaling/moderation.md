# Moderation

Module which contains logic to handle moderation based features.

Flow for joining a room with the waiting-room enabled:

```mermaid
sequenceDiagram
    participant Joinee
    participant Server
    Participant Moderator

    Joinee->>Server: Control/Join
    Server->>Joinee: Moderation/InWaitingRoom
    Server->>Moderator: Moderation/JoinedWaitingRoom(Joinee)
    Moderator->>Server: Moderation/Accept(Joinee)
    Server->>Joinee: Moderation/Accepted
    Joinee->>Server: Control/EnterRoom
    Server->>Joinee: JoinSuccess
```

## Joining the room

### JoinSuccess

When joining a room, the `join_success` control event contains the module-specific fields described below.

#### Fields

| Field                       | Type            | Always | Description                                                                           |
| --------------------------- | --------------- | ------ | ------------------------------------------------------------------------------------- |
| `raise_hands_enabled`       | `bool`          | yes    | If raise hands feature is enabled                                                     |
| `waiting_room_enabled`      | `bool`          | no     | If waiting room is enabled                                                            |
| `waiting_room_participants` | `Participant[]` | no     | A list of participants in the waiting room; see [Participant](control.md#participant) |

##### Example

For a moderator, the message would look like:

```json
{
    "waiting_room_enabled": true,
    "waiting_room_participants": [
        {
            "id": "00000000-0000-0000-0000-000000000000",
            "control": {
                ...
            },
            ...
        },
        {
            "id": "00000000-0000-0000-0000-000000000001",
            "control": {
                ...
            },
            ...
        },
        ...
    ],
    "raise_hands_enabled": true
}
```

For a non-moderator participant, the message would look like:

```json
{
    "raise_hands_enabled": true
}
```

### Joined

When joining a room, the `joined` control event sent to all other participants does not contain module-specific data.

## Commands

### Kick

Requires moderator role.

Remove a participant from the room.

#### Fields

| Field    | Type     | Required | Description                   |
| -------- | -------- | -------- | ----------------------------- |
| `action` | `enum`   | yes      | Must be `"kick"`              |
| `target` | `string` | yes      | Id of the participant to kick |

##### Example

```json
{
    "action": "kick",
    "target": "00000000-0000-0000-0000-000000000000"
}
```

---

### Ban

Requires moderator role.

Ban a user from the room. Can only be used on participants where `participation_kind` is "user". The user's user-id will
be banned from the room for the remainder of the session.

#### Fields

| Field    | Type     | Required | Description                  |
| -------- | -------- | -------- | ---------------------------- |
| `action` | `enum`   | yes      | Must be `"ban"`              |
| `target` | `string` | yes      | Id of the participant to ban |

##### Example

```json
{
    "action": "ban",
    "target": "00000000-0000-0000-0000-000000000000"
}
```

---

### SendToWaitingRoom

Requires moderator role.

Behaves like a `Kick` command and removes a participant from the room, but also implies that the frontend client should
attempt to re-join the room to end up in the waiting room.

The frontend client should enable the waiting room with the [EnableWaitingRoom](#enablewaitingroom) command before
sending this command.

Returns an [Error](#error) with variant `cannot_send_room_owner_to_waiting_room` when the target is the room owner.

#### Fields

| Field    | Type     | Required | Description                                       |
| -------- | -------- | -------- | ------------------------------------------------- |
| `action` | `enum`   | yes      | Must be `"send_to_waiting_room"`                  |
| `target` | `string` | yes      | Id of the participant to send to the waiting room |

##### Example

```json
{
    "action": "send_to_waiting_room",
    "target": "00000000-0000-0000-0000-000000000000"
}
```

---

### Debrief

Starts a debriefing.

#### Fields

| Field        | Type   | Required | Description                                        |
| ------------ | ------ | -------- | -------------------------------------------------- |
| `action`     | `enum` | yes      | Must be `"debrief"`                                |
| `kick_scope` | `enum` | yes      | Either `"guests"`, `"users_and_guests"` or `"all"` |

##### Example

```json
{
    "action": "debrief",
    "kick_scope": "guest"
}
```

---

### ChangeDisplayName

Requires moderator role.

Change the display name of a guest or phone user for the duration of the meeting.

This will trigger a [Control/Update](control.md#update) message for the targeted participant and a
[DisplayNameChanged](#displaynamechanged) is sent to all participants.

Can return [Error](#error) of kind `cannot_change_name_of_registered_users` when the targeted participant is not a guest
or dial-in user or kind `invalid_display_name` when the `new_name` is empty or longer than 100 characters.

#### Fields

| Field      | Type     | Required | Description                                                    |
| ---------- | -------- | -------- | -------------------------------------------------------------- |
| `action`   | `enum`   | yes      | Must be `"change_display_name"`                                |
| `new_name` | `string` | yes      | The new display name                                           |
| `target`   | `string` | yes      | The id of the participant that gets their display name changed |

##### Example

```json
{
    "action": "change_display_name",
    "new_name": "Foo Bar",
    "target": "00000000-0000-0000-0000-000000000000"
}
```

---

### EnableWaitingRoom

Requires moderator role.

Enable the waiting room.

#### Fields

| Field    | Type   | Required | Description                     |
| -------- | ------ | -------- | ------------------------------- |
| `action` | `enum` | yes      | Must be `"enable_waiting_room"` |

##### Example

```json
{
    "action": "enable_waiting_room"
}

```

---

### DisableWaitingRoom

Requires moderator role.

Disable the waiting room.

__NOTE:__ Participants inside the waiting room will not be automatically accepted.

#### Fields

| Field    | Type   | Required | Description                      |
| -------- | ------ | -------- | -------------------------------- |
| `action` | `enum` | yes      | Must be `"disable_waiting_room"` |

##### Example

```json
{
    "action": "disable_waiting_room"
}

```

---

### EnableRaisedHands

Requires moderator role.

Enable raising of hands by participants in the room.

#### Fields

| Field    | Type   | Required | Description                    |
| -------- | ------ | -------- | ------------------------------ |
| `action` | `enum` | yes      | Must be `"enable_raise_hands"` |

##### Example

```json
{
    "action": "enable_raise_hands"
}

```

---

### DisableRaiseHands

Requires moderator role.

Disable raising of hands by participants in the room.

__NOTE:__ Raised hands by participants will be lowered when this command is
triggered.

#### Fields

| Field    | Type   | Required | Description                     |
| -------- | ------ | -------- | ------------------------------- |
| `action` | `enum` | yes      | Must be `"disable_raise_hands"` |

##### Example

```json
{
    "action": "disable_raise_hands"
}

```

---

### Accept

Requires moderator role.

Accept a participant inside the waiting into the room.

#### Fields

| Field    | Type     | Required | Description                     |
| -------- | -------- | -------- | ------------------------------- |
| `action` | `enum`   | yes      | Must be `"accept"`              |
| `target` | `string` | yes      | Id of the participant to accept |

##### Example

```json
{
    "action": "accept",
    "target": "00000000-0000-0000-0000-000000000000"
}

```

---

### ResetRaisedHands

Requires moderator role.

Reset the raised hands of participants in the room.

#### Fields

| Field    | Type                   | Required | Description                                                                                                                        |
| -------- | ---------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `action` | `enum`                 | yes      | Must be `"reset_raised_hands"`                                                                                                     |
| `target` | `string` or `string[]` | no       | Id of one or more participants to reset the hand for. If missing, all participants in the room will have their raised hands reset. |

##### Example

Reset the hands of all participants in the room:

```json
{
    "action": "reset_raised_hands"
}
```

Reset the hand of one specific participant:

```json
{
    "action": "reset_raised_hands",
    "target": "00000000-0000-0000-0000-000000000000"
}
```

This message will not reset any hands, yet is still valid:

```json
{
    "action": "reset_raised_hands",
    "target": []
}
```

The hand of one specific participant can also be reset in a list:

```json
{
    "action": "reset_raised_hands",
    "target": ["00000000-0000-0000-0000-000000000000"]
}
```

Reset the hand of multiple participants:

```json
{
    "action": "reset_raised_hands",
    "target": [
        "00000000-0000-0000-0000-000000000000",
        "00000000-0000-0000-0000-0000000a11c3",
        "00000000-0000-0000-0000-000000000808",
    ]
}
```

---

## Events

### Kicked

Received by a participant if kicked from the room. Will be the last message before server-side websocket disconnection.

#### Fields

| Field     | Type   | Always | Description   |
| --------- | ------ | ------ | ------------- |
| `message` | `enum` | yes    | Is `"kicked"` |

##### Example

```json
{
    "message": "kicked"
}
```

---

### Banned

Received by a participant if banned from the room. Will be the last message before server-side websocket disconnection.

#### Fields

| Field     | Type   | Always | Description   |
| --------- | ------ | ------ | ------------- |
| `message` | `enum` | yes    | Is `"banned"` |

##### Example

```json
{
    "message": "banned"
}
```

---

### SentToWaitingRoom

Received by a participant if they were sent to the waiting room with the [SendToWaitingRoom](#sendtowaitingroom) Command.
The websocket connection will be kept open, but the participant state will be reset as if the participant just joined the waiting room.

#### Fields

| Field     | Type   | Always | Description                 |
| --------- | ------ | ------ | --------------------------- |
| `message` | `enum` | yes    | Is `"sent_to_waiting_room"` |

##### Example

```json
{
    "message": "sent_to_waiting_room"
}
```

---

### SessionEnded

Received by a participant if removed from the room (e.g. due to debriefing). Will be the last message before server-side websocket disconnection.

#### Fields

| Field       | Type     | Always | Description                 |
| ----------- | -------- | ------ | --------------------------- |
| `message`   | `enum`   | yes    | Is `"session_ended"`        |
| `issued_by` | `string` | yes    | Id of the issuing moderator |

---

### DebriefingStarted

Received when a debriefing has started that a participant is part of.

#### Fields

| Field       | Type     | Always | Description                 |
| ----------- | -------- | ------ | --------------------------- |
| `message`   | `enum`   | yes    | Is `"debriefing_started"`   |
| `issued_by` | `string` | yes    | Id of the issuing moderator |

---

### InWaitingRoom

Received after sending [Join](./control.md#join) in a room which has the waiting-room enabled. Once in the waiting room one must wait
until the [Accepted](#accepted) event was received. Then the room can be entered using [Control/EnterRoom](control.md#enterroom).

#### Fields

| Field     | Type   | Always | Description            |
| --------- | ------ | ------ | ---------------------- |
| `message` | `enum` | yes    | Is `"in_waiting_room"` |

##### Example

```json
{
    "message": "in_waiting_room",
}
```

---

### JoinedWaitingRoom

Received when a participant joined the waiting-room. Wraps a [Participant](./control.md#participant). Will only contain the data
of the `control` as other modules will not be initialized before the user joins the room.

#### Fields

| Field     | Type   | Always | Description                |
| --------- | ------ | ------ | -------------------------- |
| `message` | `enum` | yes    | Is `"joined_waiting_room"` |

##### Example

```json
{
    "message": "joined_waiting_room",
    "id": "00000000-0000-0000-0000-000000000000",
    "control": {
        ...
    }
}
```

---

### LeftWaitingRoom

Received when a participant left the waiting-room. Will also be received when a participant makes the transition from
waiting-room to "conference-"room.

#### Fields

| Field     | Type     | Always | Description              |
| --------- | -------- | ------ | ------------------------ |
| `message` | `enum`   | yes    | Is `"left_waiting_room"` |
| `target`  | `string` | yes    | Id of the participant    |

##### Example

```json
{
    "message": "joined_waiting_room",
    "id": "00000000-0000-0000-0000-000000000000"
}
```

---

### DisplayNameChanged

The display name of a guest participant was changed by a moderator (See [ChangeDisplayName](#changedisplayname)).

This message is received by all participants in the room.

#### Fields

| Field       | Type     | Always | Description                              |
| ----------- | -------- | ------ | ---------------------------------------- |
| `message`   | `enum`   | yes    | Is `"display_name_changed"`              |
| `target`    | `string` | yes    | Id of the targeted participant           |
| `issued_by` | `string` | yes    | The issuer of the command                |
| `old_name`  | `string` | yes    | The old name of the targeted participant |
| `new_name`  | `string` | yes    | The new name of the targeted participant |

##### Example

```json
{
    "message": "display_name_changed",
    "target": "00000000-0000-0000-0000-000000000000",
    "issued_by": "00000000-0000-0000-0000-000000000000",
    "old_name": "Alice",
    "new_name": "Bob"
}
```

---

### WaitingRoomEnabled

Received when a moderator enabled the waiting-room.

#### Fields

| Field     | Type   | Always | Description                 |
| --------- | ------ | ------ | --------------------------- |
| `message` | `enum` | yes    | Is `"waiting_room_enabled"` |

##### Example

```json
{
    "message": "waiting_room_enabled"
}
```

---

### WaitingRoomDisabled

Received when a moderator disabled the waiting-room.

#### Fields

| Field     | Type   | Always | Description                  |
| --------- | ------ | ------ | ---------------------------- |
| `message` | `enum` | yes    | Is `"waiting_room_disabled"` |

##### Example

```json
{
    "message": "waiting_room_disabled"
}
```

---

### RaiseHandsEnabled

Received when a moderator enabled raising of hands.

#### Fields

| Field       | Type     | Always | Description                 |
| ----------- | -------- | ------ | --------------------------- |
| `message`   | `enum`   | yes    | Is `"raise_hands_enabled"`  |
| `issued_by` | `string` | yes    | Id of the issuing moderator |

##### Example

```json
{
    "message": "raise_hands_enabled",
    "issued_by": "00000000-0000-0000-0000-000000000000"
}
```

---

### RaiseHandsDisabled

Received when a moderator disabled raising of hands.

#### Fields

| Field       | Type     | Always | Description                 |
| ----------- | -------- | ------ | --------------------------- |
| `message`   | `enum`   | yes    | Is `"raise_hands_disabled"` |
| `issued_by` | `string` | yes    | Id of the issuing moderator |

##### Example

```json
{
    "message": "raise_hands_disabled",
    "issued_by": "00000000-0000-0000-0000-000000000000"
}
```

---

### Accepted

Can only be received while in the waiting room. A moderator accepted this you into the room, which can now entered using
[Control/EnterRoom](control.md#enterroom).

#### Fields

| Field     | Type   | Always | Description     |
| --------- | ------ | ------ | --------------- |
| `message` | `enum` | yes    | Is `"accepted"` |

##### Example

```json
{
    "message": "accepted",

}
```

---

### Error

#### Fields

| Field     | Type   | Always | Description                           |
| --------- | ------ | ------ | ------------------------------------- |
| `message` | `enum` | yes    | Is `"error"`                          |
| `error`   | `enum` | yes    | Variant of the error, see table below |

| Error                                    | Description                                                                                        |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `cannot_ban_guest`                       | Issued when the [`Ban`](#ban) command targets a guest                                              |
| `cannot_send_room_owner_to_waiting_room` | Issued when the [`SendToWaitingRoom`](#sendtowaitingroom) command targets the room owner           |
| `cannot_change_name_of_registered_users` | Issued when the [`ChangeDisplayName`](#changedisplayname) command targets a registered user        |
| `invalid_display_name`                   | Issued when the [`ChangeDisplayName`](#changedisplayname) command contains an invalid display name |
| `permission_denied`                      | Issued when the sender of a command wasn't a moderator                                             |

##### Example

```json
{
    "message": "error",
    "error": "cannot_ban_guest"
}
```

---

### RaisedHandResetByModerator

Received when a moderator lowered the hand of the participant.

#### Fields

| Field       | Type     | Always | Description                           |
| ----------- | -------- | ------ | ------------------------------------- |
| `message`   | `enum`   | yes    | Is `"raised_hand_reset_by_moderator"` |
| `issued_by` | `string` | yes    | Id of the issuing moderator           |

##### Example

```json
{
    "message": "raised_hand_reset_by_moderator",
    "issued_by": "00000000-0000-0000-0000-000000000000"
}
```
