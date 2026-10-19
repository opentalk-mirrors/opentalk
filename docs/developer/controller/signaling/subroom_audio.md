# Subroom Audio

This module implements signaling that allows participants to manage a group of whisper participants. Participants in a
whisper group gain access to an audio-only livekit room, where they can talk without being heard by other participants
in the conference.

The implementing frontend, still has to do most of the livekit management associated with this feature. This module only
creates the livekit room, the tokens to join that room and tracks the whisper group state to enforce some behavior.

The `WhisperId` that is returned in various events is also the name of the livekit room.

After a livekit access token has been provided through the [WhisperGroupCreated](#whispergroupcreated) or
[WhisperToken](#whispertoken) event, the frontend has to connect to the whisper room by itself. The tokens are
restricted to audio only, and are revoked when the participant leaves the whisper group.

## Commands

### CreateWhisperGroup

Creates a new whisper group with the targeted participants. The group creator is implicitly contained in the group.

#### Fields

| Field              | Type            | Required | Description                      |
| ------------------ | --------------- | -------- | -------------------------------- |
| `action`           | `enum`          | yes      | Must be `"create_whisper_group"` |
| `participants_ids` | `Participant[]` | yes      | Ids of participants to invite    |

##### Example

```json
{
    "action": "create_whisper_group",
    "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
}
```

#### Response

[WhisperGroupCreated](#whispergroupcreated) for the creator and [WhisperInvite](#whisperinvite) for the invitees.

---

### InviteToWhisperGroup

Invite participants to an existing whisper group.

#### Fields

| Field             | Type            | Required | Description                         |
| ----------------- | --------------- | -------- | ----------------------------------- |
| `action`          | `enum`          | yes      | Must be `"invite_to_whisper_group"` |
| `whisper_id`      | `string`        | yes      | The id of the whisper group         |
| `participant_ids` | `Participant[]` | yes      | Ids of participants to invite       |

##### Example

```json
{
    "action": "invite_to_whisper_group",
    "whisper_id": "00000000-0000-0000-0000-000000000000",
    "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
}
```

#### Response

[ParticipantsInvited](#participantsinvited) for the creator and existing participants and [WhisperInvite](#whisperinvite)
for the new invitees.

---

### AcceptWhisperInvite

Accept an invite to a whisper group.

#### Fields

| Field        | Type     | Required | Description                       |
| ------------ | -------- | -------- | --------------------------------- |
| `action`     | `enum`   | yes      | Must be `"accept_whisper_invite"` |
| `whisper_id` | `string` | yes      | The id of the whisper group       |

##### Example

```json
{
    "action": "accept_whisper_invite",
    "whisper_id": "00000000-0000-0000-0000-000000000000"
}
```

#### Response

[WhisperToken](#whispertoken) to the accepting participant, containing the access token for the
whisper room. Other participants in the whisper group receive a [WhisperInviteAccepted](#whisperinviteaccepted) as a
notification that another participant joined the room.

---

### DeclineWhisperInvite

Decline an invitation to a whisper group.

#### Fields

| Field        | Type     | Required | Description                        |
| ------------ | -------- | -------- | ---------------------------------- |
| `action`     | `enum`   | yes      | Must be `"decline_whisper_invite"` |
| `whisper_id` | `string` | yes      | The id of the whisper group        |

##### Example

```json
{
    "action": "decline_whisper_invite",
    "whisper_id": "00000000-0000-0000-0000-000000000000"
}
```

#### Response

Other participants in the whisper group receive a [WhisperInviteDeclined](#whisperinvitedeclined).

---

### KickWhisperParticipants

Remove a participant from the whisper group, only the creator of the group can kick participants.

#### Fields

| Field             | Type            | Required | Description                                      |
| ----------------- | --------------- | -------- | ------------------------------------------------ |
| `action`          | `enum`          | yes      | Must be `"kick_whisper_participants"`            |
| `whisper_id`      | `string`        | yes      | The id of the whisper group                      |
| `participant_ids` | `Participant[]` | yes      | Ids of participants to be removed from the group |

##### Example

```json
{
    "action": "kick_whisper_participants",
    "whisper_id": "00000000-0000-0000-0000-000000000000",
    "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
}
```

#### Response

The kicked participants receive a [Kicked](#kicked) message. Other participants in the whisper group receive a
[LeftWhisperGroup](#leftwhispergroup) for each participant that has been kicked.

---

### LeaveWhisperGroup

Leave the whisper group. The participant will get removed from the livekit whisper room.

The group gets disbanded when the last participant leaves the group.

#### Fields

| Field        | Type     | Required | Description                     |
| ------------ | -------- | -------- | ------------------------------- |
| `action`     | `enum`   | yes      | Must be `"leave_whisper_group"` |
| `whisper_id` | `string` | yes      | The id of the whisper group     |

##### Example

```json
{
    "action": "leave_whisper_group",
    "whisper_id": "00000000-0000-0000-0000-000000000000"
}
```

#### Response

Other participants in the whisper group receive a [LeftWhisperGroup](#leftwhispergroup).

---

## Events

### WhisperGroupCreated

Direct response to [CreateWhisperGroup](#createwhispergroup).

The participants in the list have a state associated with them, see [WhisperParticipant](#whisperparticipant) for the
type structure.

#### Fields

| Field          | Type                   | Required | Description                                                                                  |
| -------------- | ---------------------- | -------- | -------------------------------------------------------------------------------------------- |
| `message`      | `enum`                 | yes      | Is `"whisper_group_created"`                                                                 |
| `whisper_id`   | `string`               | yes      | The id of the whisper group                                                                  |
| `participants` | `WhisperParticipant[]` | yes      | List of invited participants and their state (see [WhisperParticipant](#whisperparticipant)) |
| `token`        | `string`               | yes      | The livekit access token that allows the participant to join the whisper room                |

##### Example

```json
{
  "message": "whisper_group_created",
  "whisper_id": "00000000-0000-0000-0000-000000000000",
  "token": "<jwt-token>",
  "participants": [
    {
      "participant_id": "00000000-0000-0000-0000-000000000000",
      "state": "creator"
    },
    {
      "participant_id": "00000000-0000-0000-0000-000000000001",
      "state": "accepted"
    },
    {
      "participant_id": "00000000-0000-0000-0000-000000000002",
      "state": "invited"
    }
  ]
}
```

---

### WhisperInvite

Received by participants that were invited through [CreateWhisperGroup](#createwhispergroup) or
[InviteToWhisperGroup](#invitetowhispergroup).

The participants in the list have a state associated with them, see [WhisperParticipant](#whisperparticipant) for the
type structure.

#### Fields

| Field          | Type                   | Required | Description                                                                                  |
| -------------- | ---------------------- | -------- | -------------------------------------------------------------------------------------------- |
| `message`      | `enum`                 | yes      | Is `"whisper_invite"`                                                                        |
| `whisper_id`   | `string`               | yes      | The id of the whisper group                                                                  |
| `issuer`       | `string`               | yes      | The participant id of the invite issuer                                                      |
| `participants` | `WhisperParticipant[]` | yes      | List of invited participants and their state (see [WhisperParticipant](#whisperparticipant)) |

##### Example

```json
{
  "message": "whisper_invite",
  "whisper_id": "00000000-0000-0000-0000-000000000000",
  "issuer": "00000000-0000-0000-0000-000000000000",
  "participants": [
    {
      "participant_id": "00000000-0000-0000-0000-000000000000",
      "state": "invited"
    },
    {
      "participant_id": "00000000-0000-0000-0000-000000000000",
      "state": {
        "accepted": {
          "track_id": "00000000-0000-0000-0000-000000000000"
        }
      }
    }
  ]
}
```

---

### WhisperToken

A direct response to [AcceptWhisperInvite](#acceptwhisperinvite). Contains the livekit access token to the whisper room.

#### Fields

| Field        | Type     | Required | Description                                                                   |
| ------------ | -------- | -------- | ----------------------------------------------------------------------------- |
| `message`    | `enum`   | yes      | Is `"whisper_token"`                                                          |
| `whisper_id` | `string` | yes      | The id of the whisper group                                                   |
| `token`      | `string` | yes      | The livekit access token that allows the participant to join the whisper room |

##### Example

```json
{
  "message": "whisper_token",
  "whisper_id": "00000000-0000-0000-0000-000000000000",
  "token": "<jwt-token>"
}
```

---

### ParticipantsInvited

Received by all participants when a new set of participants were invited through [InviteToWhisperGroup](#invitetowhispergroup).

#### Fields

| Field             | Type            | Required | Description                      |
| ----------------- | --------------- | -------- | -------------------------------- |
| `message`         | `enum`          | yes      | Is `"participants_invited"`      |
| `whisper_id`      | `string`        | yes      | The id of the whisper group      |
| `participant_ids` | `Participant[]` | yes      | The list of invited participants |

##### Example

```json
{
  "message": "participants_invited",
  "whisper_id": "00000000-0000-0000-0000-000000000000",
  "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
}
```

---

### WhisperInviteAccepted

Received by all participants in the whisper group when another invitee accepts the group invite.

#### Fields

| Field            | Type          | Required | Description                                    |
| ---------------- | ------------- | -------- | ---------------------------------------------- |
| `message`        | `enum`        | yes      | Is `"whisper_invite_accepted"`                 |
| `whisper_id`     | `string`      | yes      | The id of the whisper group                    |
| `participant_id` | `Participant` | yes      | The participant that accepted the group invite |

##### Example

```json
{
  "message": "whisper_invite_accepted",
  "whisper_id": "00000000-0000-0000-0000-000000000000",
  "participant_id": "00000000-0000-0000-0000-000000000000"
}
```

---

### WhisperInviteDeclined

Received by all participants in the whisper group when another invitee declines the group invite.

#### Fields

| Field            | Type          | Required | Description                              |
| ---------------- | ------------- | -------- | ---------------------------------------- |
| `message`        | `enum`        | yes      | Is `"whisper_invite_declined"`           |
| `whisper_id`     | `string`      | yes      | The id of the whisper group              |
| `participant_id` | `Participant` | yes      | The participant that declined the invite |

##### Example

```json
{
  "message": "whisper_invite_declined",
  "whisper_id": "00000000-0000-0000-0000-000000000000",
  "participant_id": "00000000-0000-0000-0000-000000000000"
}
```

---

### Kicked

Received by a participant when they get kicked from the whisper group. The whisper group will be inaccessible after receiving
this message.

#### Fields

| Field        | Type     | Required | Description                 |
| ------------ | -------- | -------- | --------------------------- |
| `message`    | `enum`   | yes      | Is `"kicked"`               |
| `whisper_id` | `string` | yes      | The id of the whisper group |

##### Example

```json
{
  "message": "kicked",
  "whisper_id": "00000000-0000-0000-0000-000000000000",
}
```

---

### LeftWhisperGroup

Received when another participant leaves the whisper group either through [LeaveWhisperGroup](#leavewhispergroup) or
gets kicked with the [Kicked](#kicked) command.

#### Fields

| Field        | Type     | Required | Description                 |
| ------------ | -------- | -------- | --------------------------- |
| `message`    | `enum`   | yes      | Is `"left_whisper_group"`   |
| `whisper_id` | `string` | yes      | The id of the whisper group |

##### Example

```json
{
  "message": "left_whisper_group",
  "whisper_id": "00000000-0000-0000-0000-000000000000",
}
```

---

### Error

#### Fields

| Field             | Type            | Always | Description                                                                              |
| ----------------- | --------------- | ------ | ---------------------------------------------------------------------------------------- |
| `message`         | `enum`          | yes    | Is `"error"`                                                                             |
| `error`           | `enum`          | yes    | Exhaustive list of error strings, see table below                                        |
| `participant_ids` | `Participant[]` | non    | Only present when error is `invalid_participant_targets`. A List of invalid participants |

| Error                         | Description                                                        |
| ----------------------------- | ------------------------------------------------------------------ |
| `invalid_whisper_id`          | The provided whisper id does not exist                             |
| `already_accepted`            | The participant already accepted an invite to this group           |
| `insufficient_permissions`    | The requesting user has insufficient permissions for the operation |
| `empty_participant_list`      | The provided participant list is empty                             |
| `invalid_participant_targets` | The targeted participants do not exist                             |
| `livekit_unavailable`         | The livekit service is unavailable                                 |
| `not_invited`                 | The requesting participant is not invited to the whisper group     |

##### Example

```json
{
    "message":"error",
    "error":"invalid_whisper_id"
}
```

```json
{
    "message":"error",
    "error":"insufficient_permissions"
}
```

```json
{
    "message":"error",
    "error":"empty_participant_list"
}
```

```json
{
    "message":"error",
    "error":"invalid_participant_targets",
    "participant_ids": ["00000000-0000-0000-0000-000000000123"]
}
```

---

## Structural Types

### WhisperParticipant

A representation of a whisper participant and their invitation state. Used in [WhisperGroupCreated](#whispergroupcreated)
and [WhisperInvite](#whisperinvite)

| Field            | Type          | Required | Description                                                        |
| ---------------- | ------------- | -------- | ------------------------------------------------------------------ |
| `participant_id` | `Participant` | yes      | The participant id                                                 |
| `state`          | `enum`        | yes      | The state of the participant in the whisper group, see table below |

| State      | Description                                                  |
| ---------- | ------------------------------------------------------------ |
| `creator`  | The initiator of the whisper group gets the `creator` state  |
| `invited`  | The participant has been invited to the whisper group        |
| `accepted` | The participant has accepted the invite to the whisper group |
