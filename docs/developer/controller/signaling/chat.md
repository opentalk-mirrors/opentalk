# Chat

The chat module allows users to send messages to either the whole
conference room, or to individual users.

## Joining the room

### JoinSuccess

When joining a room, the `join_success` control event contains the module-specific fields described below.

#### Fields

| Field                          | Type               | Always | Description                                                            |
| ------------------------------ | ------------------ | ------ | ---------------------------------------------------------------------- |
| `enabled`                      | `bool`             | yes    | When true, the chat is enabled                                         |
| `room_history`                 | `StoredMessage[]`  | yes    | Chat history for the room                                              |
| `groups_history`               | `GroupHistory[]`   | yes    | Chat history for each group                                            |
| `private_history`              | `PrivateHistory[]` | yes    | Chat history for a participant's private conversations                 |
| `last_seen_timestamp_global`   | `string`           | no     | Last seen timestamp for the global chat                                |
| `last_seen_timestamps_private` | `map`              | no     | Last seen timestamps for private chats. Map key is the participant id. |
| `last_seen_timestamps_group`   | `map`              | no     | Last seen timestamps for group chats. Map key is the group name.       |

##### Example

```json
    ...
    "room_history": [
        {
            "source": "00000000-0000-0000-0000-000000000000",
            "scope": "global",
            "content": "Hello all!",
            "timestamp": "2023-01-13T12:37:08Z"
        },
        ...
    ],
    "groups_history": [
        {
            "history": [
                {
                    "source": "00000000-0000-0000-0000-000000000000",
                    "scope": "group",
                    "target": "management",
                    "content": "Hello managers!",
                    "timestamp": "2023-01-13T12:37:08Z"
                }
            ],
            "name": "/OpenTalk/OU/Backend"
        },
        ...
    ],
    "private_history": [
        {
            "history": [
                {
                    "source": "00000000-0000-0000-0000-000000000000",
                    "scope": "private",
                    "target": "00000000-0000-0000-0000-000000000001",
                    "content": "Hello private chat correspondent!",
                    "timestamp": "2023-01-13T12:37:08Z"
                }
            ],
            "correspondent": "00000000-0000-0000-0000-000000000000"
        },
        ...
    ],
    "last_seen_timestamp_global": "2023-01-03T12:34:56Z",
    "last_seen_timestamps_private": {
        "00000000-0000-0000-0000-000000000000": "2023-01-02T11:22:33Z"
    },
    "last_seen_timestamps_group": {
        "/OpenTalk/OU/Backend": "2023-01-02T12:11:11Z",
        "/OpenTalk/Role/Developer": "2023-01-02T12:10:23Z"
    }
    ...
```

### Joined

When joining a room, the `joined` control event sent to all other participants contains the module-specific fields described below.

#### Fields

| Field        | Type       | Always | Description                         |
| ------------ | ---------- | ------ | ----------------------------------- |
| `groups`     | `string[]` | yes    | Groups belonging to the participant |

##### Example

```json
    ...
    "groups": [
        "/OpenTalk/OU/Backend",
        "/OpenTalk/Role/Developer",
        ...
    ]
    ...

```

## Commands

### EnableChat

Allows a moderator to enable the chat if it is currently disabled.

#### Fields

| Field    | Type   | Required | Description             |
| -------- | ------ | -------- | ----------------------- |
| `action` | `enum` | yes      | Must be `"enable_chat"` |

##### Example

```json
{
    "action": "enable_chat"
}
```

---

### DisableChat

Allows a moderator to disable the chat if it is currently enabled.

#### Fields

| Field    | Type   | Required | Description              |
| -------- | ------ | -------- | ------------------------ |
| `action` | `enum` | yes      | Must be `"disable_chat"` |

##### Example

```json
{
    "action": "disable_chat"
}
```

---

### SendMessage

Send a message to either the conference room (global message), a group or a
specific user (direct message).

#### Fields

| Field     | Type     | Required | Description                                                            |
| --------- | -------- | -------- | ---------------------------------------------------------------------- |
| `action`  | `enum`   | yes      | Must be `"send_message"`                                               |
| `scope`   | `enum`   | yes      | Either `"global"`, `"group"` or `"private"`                            |
| `target`  | `string` | no       | Needed if `scope` is `"group"` or `"private"`. Participant id or group |
| `content` | `string` | yes      | The message content                                                    |

##### Example

```json
{
    "action": "send_message",
    "scope": "global",
    "content": "Hello all!"
}
```

```json
{
    "action": "send_message",
    "scope": "group",
    "target": "management",
    "content": "Hello managers!"
}
```

```json
{
    "action": "send_message",
    "scope": "private",
    "target": "00000000-0000-0000-0000-000000000000",
    "content": "Hello Bob!"
}
```

---

### ClearHistory

Allows a moderator to clear the global chat history of the conference room.

#### Fields

| Field    | Type   | Required | Description               |
| -------- | ------ | -------- | ------------------------- |
| `action` | `enum` | yes      | Must be `"clear_history"` |

##### Example

```json
{
    "action": "clear_history"
}
```

---

### SetLastSeenTimestamp

Set the last seen timestamp for either global chat messages, group or private
messages with a specific participant.

The values that are set in `SetLastSeenTimestamp`, will be included in the `chat` field of
`module_data` inside the [JoinSuccess](control.md#joinsuccess) message when
reconnecting to the same room after leaving.

#### Fields

| Field       | Type     | Required                               | Description                                                |
| ----------- | -------- | -------------------------------------- | ---------------------------------------------------------- |
| `action`    | `enum`   | yes                                    | Must be `"set_last_seen_timestamp"`                        |
| `scope`     | `enum`   | yes                                    | Either `"global"`, `"group"` or `"private"`                |
| `target`    | `string` | If `scope` is `"group"` or `"private"` | Participant id or group                                    |
| `timestamp` | `string` | yes                                    | The timestamp when the messages in the chat were last read |

##### Example

```json
{
    "action": "set_last_seen_timestamp",
    "scope": "global",
    "timestamp": "2022-10-22T11:22:33Z"
}
```

```json
{
    "action": "set_last_seen_timestamp",
    "scope": "group",
    "target": "/OpenTalk/OU/Backend",
    "timestamp": "2022-10-22T11:22:33Z"
}
```

```json
{
    "action": "set_last_seen_timestamp",
    "scope": "private",
    "target": "00000000-0000-0000-0000-000000000000",
    "timestamp": "2022-10-22T11:22:33Z"
}
```

## Events

### MessageSent

A message has been sent to either the global chat or directly to the participant.

#### Fields

| Field       | Type     | Always | Description                                                          |
| ----------- | -------- | ------ | -------------------------------------------------------------------- |
| `message`   | `enum`   | yes    | Is `"message_sent"`                                                  |
| `source`    | `string` | yes    | Id of the participant who sent the message                           |
| `scope`     | `enum`   | yes    | Either `"global"`, `"group"` or `"private"`                          |
| `target`    | `string` | no     | Only if `scope` is `"group"` or `"private"`. Participant id or group |
| `content`   | `string` | yes    | The message content                                                  |

##### Example

```json
{
    "message": "message_sent",
    "source": "00000000-0000-0000-0000-000000000000",
    "scope": "global",
    "content": "Hello all!"
}
```

```json
{
    "message": "message_sent",
    "source": "00000000-0000-0000-0000-000000000000",
    "scope": "group",
    "target": "management",
    "content": "Hello managers!"
}
```

```json
{
    "message": "message_sent",
    "source": "00000000-0000-0000-0000-000000000000",
    "scope": "private",
    "target": "00000000-0000-0000-0000-0000deadbeef",
    "content": "Hello Bob!"
}
```

### Error

Received when something went wrong processing messages sent to the server.

#### Fields

| Field     | Type   | Always | Description                                       |
| --------- | ------ | ------ | ------------------------------------------------- |
| `message` | `enum` | yes    | Is `"error"`                                      |
| `error`   | `enum` | yes    | Exhaustive list of error strings, see table below |

| Error                      | Description                                                     |
| -------------------------- | --------------------------------------------------------------- |
| `chat_disabled`            | A message was sent while the chat was disabled                  |
| `insufficient_permissions` | A moderator action was attempted by a non-moderator participant |

```json
{
    "message": "error",
    "error": "chat_disabled"
}
```
