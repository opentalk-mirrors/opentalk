# Legal Vote

---

## Terminology

- __User__ - A user is an account inside OpenTalk, a single user may be inside a room as multiple participants.
- __Participant__ - A unique connection to a room. Participants may be a guest or logged-in users.

## Overview

The legal vote module allows users to participate in a legally sound vote. When a vote is started, a selection of
allowed participants can cast the vote options `Yes`, `No` and `Abstain` on a defined voting topic. An allowed
participant cannot be a guest, as each participant needs to have an underlying user id to cast a vote.

While a vote is active, the occurring vote events are saved in a `vote protocol` which is moved to the database once the
vote is complete. This provides a foundation to backtrack results, votes and errors after a vote has ended. The result
and protocol of a vote can be accessed in redis for the lifetime of the room where the vote had taken place in.

The state of a vote is held in a redis instance. There can only be one active vote at a time, every completed vote gets
moved to the `vote history` for later access. Any operation that accesses more than one redis key is done with a Lua
script in order to make those operations atomic.

## Vote kind

The voting procedure may follow different patterns depending on their kind. In order to choose the vote kind, the corresponding
identifier needs to be set in the [Start](#start) message.

### Live roll call

Identifier: `"live_roll_call"`

In a live roll call, every vote that is handed in gets published to all room participants immediately. This allows everybody
to follow the current state of the voting procedure live, immediately revealing who voted what. The report will contain a list
of user ids and the vote options they chose.

### Roll call

Identifier: `"roll_call"`

A roll call is private while it is running, while the details get published once it is finished. No updates will be sent out
during the voting procedure, but votes are counted by the server. The report will contain a list of users and the vote options
they chose.

### Pseudonymous

Identifier: `"pseudonymous"`

A pseudonymous vote allows the users to keep their vote option private. Each user gets a single use token which is consumed
when the the vote option is counted. The token is revealed to the user, who should keep it private. The report will contain a
list of tokens and the vote options that were handed in with them. This allows every user to verify that their vote has been
counted correctly. Because the tokens are published with the association of the chosen vote option, this is only pseudonymous,
not anonymous.

## Joining the room

### JoinSuccess

When joining a room, the `join_success` control event contains the module-specific fields described below.

#### Fields

| Field   | Type            | Always | Description                       |
| ------- | --------------- | ------ | --------------------------------- |
| `votes` | `VoteSummary[]` | yes    | A list of current and past votes. |

__`VoteSummary` fields__:

| Field                  | Type       | Required | Description                                                                                   |
| ---------------------- | ---------- | -------- | --------------------------------------------------------------------------------------------- |
| `kind`                 | `enum`     | yes      | The exhaustive list of `kind` can be found in section [Vote Kind](#vote-kind).                |
| `initiator_id`         | `string`   | yes      | Id of the participant which started the vote.                                                 |
| `legal_vote_id`        | `string`   | yes      | Id of the vote.                                                                               |
| `start_time`           | `string`   | yes      | RFC 3339 timestamp when the vote started.                                                     |
| `max_votes`            | `int`      | yes      | The maximum number of possible votes.                                                         |
| `name`                 | `string`   | yes      | General name of the vote.                                                                     |
| `subtitle`             | `string`   | no       | A subtitle for the vote                                                                       |
| `topic`                | `string`   | no       | Detailed topic that will be voted on.                                                         |
| `allowed_participants` | `string[]` | yes      | An array of participant ids, where each contained participant is allowed to cast a vote.      |
| `enable_abstain`       | `bool`     | yes      | Enable/Disable the 'Abstain' option on this vote.                                             |
| `auto_close`           | `bool`     | yes      | When set, the vote will automatically close when every allowed participant casted a vote.     |
| `create_pdf`           | `bool`     | yes      | Automatically create a protocol PDF when the vote ends.                                       |
| `duration`             | `int`      | no       | Duration of the vote in seconds, counting from the `start_time`.                              |
| `token`                | `string`   | no       | Optional. Only users who participate in the voting procedure receive a token.                 |
| `state`                | `string`   | yes      | The state of the vote. Valid values are `"started"`, `"finished"`, `"canceled"`, `"invalid"`. |
| `end_time`             | `string`   | no       | RFC 3339 timestamp when the vote ended.                                                       |

When `state` is `"finished"`:

| Field           | Type     | Required                        | Description                                                                                 |
| --------------- | -------- | ------------------------------- | ------------------------------------------------------------------------------------------- |
| `stop_kind`     | `enum`   | yes                             | Describes how the vote finished. Valid values are `"by_user"`, `"auto"`, `"expired"`.       |
| `stopped_by`    | `string` | no                              | The id of the participant who stopped the vote. Only present if `stop_kind` is `"by_user"`. |
| `yes`           | `int`    | yes                             | Number of `"yes"` votes                                                                     |
| `no`            | `int`    | yes                             | Number of `"no"` votes                                                                      |
| `abstain`       | `int`    | when `enable_abstain` is `true` | Number of `"abstain"` votes                                                                 |
| `voting_record` | `map`    | yes                             | Mapping of participant or token including their vote option                                 |

When `state` is `"canceled"`:

| Field    | Type     | Required | Description                                                      |
| -------- | -------- | -------- | ---------------------------------------------------------------- |
| `issuer` | `string` | yes      | The id of the participant who canceled the vote                  |
| `reason` | `string` | yes      | Either `"room_destroyed"`, `"initiator_left"` or `"custom"`      |
| `custom` | `string` | no       | A custom cancel reason, only present when `reason` is `"custom"` |

When `state` is `"invalid"`:

| Field    | Type   | Required | Description                                                                     |
| -------- | ------ | -------- | ------------------------------------------------------------------------------- |
| `reason` | `enum` | yes      | `"abstain_disabled"`, `"vote_count_inconsistent"`, or `"protocol_inconsistent"` |

##### Example

Stopped with valid results on a vote which reveals the users:

```json
[
    {
        "legal_vote_id": "00000000-0000-0000-0000-000000000123",
        "kind": "live_roll_call",
        "initiator_id":  "00000000-0000-0000-0000-000000000001",
        "start_time": "1970-01-01T00:00:00Z",
        "max_votes": 5,
        "name": "Yes or no",
        "subtitle": "Choose either yes or no",
        "allowed_participants": [
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-000000000002",
            "00000000-0000-0000-0000-000000000003",
            "00000000-0000-0000-0000-000000000004",
            "00000000-0000-0000-0000-000000000005"
        ],
        "enable_abstain": true,
        "auto_close": true,
        "create_pdf": true,
        "duration": 300,
        "state": "finished",
        "stop_kind": "auto",
        "yes": 1,
        "no": 2,
        "abstain": 2,
        "voting_record": {
            "00000000-0000-0000-0000-000000000001": "yes",
            "00000000-0000-0000-0000-000000000002": "no",
            "00000000-0000-0000-0000-000000000003": "abstain",
            "00000000-0000-0000-0000-000000000004": "abstain",
            "00000000-0000-0000-0000-000000000005": "no"
        },
        "end_time": "1970-01-01T00:05:00Z"
    },
    {
        "legal_vote_id": "00000000-0000-0000-0000-000000000456",
        "kind": "roll_call",
        "initiator_id":  "00000000-0000-0000-0000-000000000001",
        "start_time": "1970-01-01T01:00:00Z",
        "max_votes": 3,
        "name": "Vote Test",
        "subtitle": "Yes or No?",
        "allowed_participants": [
            "00000000-0000-0000-0000-000000000001",
            "00000000-0000-0000-0000-000000000002",
            "00000000-0000-0000-0000-000000000003"
        ],
        "enable_abstain": false,
        "auto_close": false,
        "create_pdf": true,
        "timezone": "CET",
        "duration": 60,
        "state": "canceled",
        "issued_by":  "00000000-0000-0000-0000-000000000001",
        "reason":  "initiator_left"
    }
]
```

### Joined

When joining a room, the `joined` control event sent to all other participants contains the module-specific fields described below.

## Commands

Commands are issued by a participant to start or interact with a vote.

### Start

The `Start` message can be sent by a moderator to start a new legal vote.

#### Fields

| Field                  | Type       | Required | Validation        | Description                                                                                |
| ---------------------- | ---------- | -------- | ----------------- | ------------------------------------------------------------------------------------------ |
| `action`               | `enum`     | yes      |                   | Must be `"start"`.                                                                         |
| `kind`                 | `enum`     | yes      |                   | The exhaustive list of `kind` can be found in section [Vote Kind](#vote-kind).             |
| `name`                 | `string`   | yes      | max 150 chars     | General name of the vote                                                                   |
| `subtitle`             | `string`   | no       | max 255 chars     | A subtitle for the vote                                                                    |
| `topic`                | `string`   | no       | max 500 chars     | Detailed topic that will be voted on.                                                      |
| `allowed_participants` | `string[]` | yes      | min 1 participant | An array of participant ids, where each contained participant is allowed to cast a vote.   |
| `enable_abstain`       | `bool`     | yes      |                   | Enable/Disable the 'Abstain' option on this vote                                           |
| `auto_close`           | `bool`     | yes      |                   | When set, the vote will automatically close when every allowed participant casted a vote.  |
| `create_pdf`           | `bool`     | yes      |                   | Automatically create a protocol PDF when the vote ends.                                    |
| `timezone`             | `string`   | no       | max 150 chars     | Timezone used in the protocol, defaults to UTC, IANA format, e.g."CET" or "Europe/Vienna". |
| `duration`             | `int`      | no       | min 5 seconds     | Duration of the vote in seconds.                                                           |

##### Example

```json
{
    "action": "start",
    "kind": "roll_call",
    "name": "Vote Test",
    "topic": "Yes or No?",
    "allowed_participants": [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "00000000-0000-0000-0000-000000000003"
    ],
    "enable_abstain": false,
    "auto_close": false,
    "create_pdf": true,
    "timezone": "CET",
    "duration": 60
}
```

#### Response

A [Started](#started) message is sent to all participants that are currently in the room.

---

### Stop

Stop the currently active vote. Will only succeed when the issuer is the vote initiator.

#### Fields

| Field           | Type     | Required | Description                    |
| --------------- | -------- | -------- | ------------------------------ |
| `action`        | `enum`   | yes      | Must be `"stop"`.              |
| `legal_vote_id` | `string` | yes      | The vote that shall be stopped |

##### Example

```json
{
    "action": "stop",
    "legal_vote_id": "00000000-0000-0000-0000-000000000000"
}
```

#### Response

A [Stopped](#stopped) message is sent to all participants that are currently in the room.

---

### Cancel

Cancel the currently active vote when the provided `legal_vote_id` matches. This command may only be issued by a moderator.
The vote protocol will still be saved in the database and in the room-vote-history, but the vote results should be handled as invalid.

This command may be triggered by the controller itself when an invalid state or error was detected.
See [Canceled](#canceled) for more details on which server events may cause this.

#### Fields

| Field           | Type     | Required | Validation    | Description                      |
| --------------- | -------- | -------- | ------------- | -------------------------------- |
| `action`        | `enum`   | yes      |               | Must be `"cancel"`.              |
| `legal_vote_id` | `string` | yes      |               | The vote that shall be canceled. |
| `reason`        | `string` | yes      | max 255 chars | The reason for the cancel.       |

##### Example

```json
{
    "action": "cancel",
    "vote_id": "00000000-0000-0000-0000-000000000000",
    "reason": "A very descriptive reason",
}
```

#### Response

A [Canceled](#canceled) message is sent to all participants that are currently in the room.

---

### Vote

Cast a vote on the specified `legal_vote_id`. Each user is allowed to only vote once.

#### Fields

| Field           | Type     | Required | Description                                                                                       |
| --------------- | -------- | -------- | ------------------------------------------------------------------------------------------------- |
| `action`        | `enum`   | yes      | Must be `"vote"`.                                                                                 |
| `legal_vote_id` | `string` | yes      | The vote that shall be voted on.                                                                  |
| `option`        | `enum`   | yes      | The chosen vote option, may be `"yes"`, `"no"` or `"abstain"` when the abstain option is enabled. |
| `token`         | `string` | yes      | The token that was handed to the user with the [Started](#started) message.                       |

##### Example

```json
{
    "action": "vote",
    "legal_vote_id": "00000000-0000-0000-0000-000000000000",
    "option": "yes",
    "token": "2QNav7b3FJw"
}
```

#### Response

When the vote is successful, a [Voted](#voted) message is sent to each participant that is logged in under the same user id.

When the vote failed, a [Voted](#voted) response is sent to the issuer.

---

### ReportIssue

Report an issue to the vote creator while the vote is active.

Can be sent by any vote participant during the vote. These events will be saved and displayed in the vote protocol.

#### Fields

| Field           | Type     | Required | Description                                                                        |
| --------------- | -------- | -------- | ---------------------------------------------------------------------------------- |
| `action`        | `enum`   | yes      | Must be `"report_issue"`.                                                          |
| `legal_vote_id` | `string` | yes      | The ID of the related legal vote                                                   |
| `kind`          | `enum`   | no       | Either `"audio"`, `"video"` or `"screenshare"`.                                    |
| `description`   | `string` | no       | An optional message to the vote creator. Is mandatory when no `"kind"` is provided |

#### Example

```json
{
    "action": "report_issue",
    "kind": "audio",
    "description": "Hello, my audio is not working :("
}
```

```json
{
    "action": "report_issue",
    "description": "Hello, something else is not working :("
}
```

#### Response

No response is sent to the issuer. The moderator will receive a [`ReportedIssue`](#reportedissue) message.

---

### GeneratePdf

Generate a PDF of the protocol of the specified `vote_id`. Only passed votes can be generated as a PDF document.

#### Fields

| Field      | Type     | Required | Description                                                                                |
| ---------- | -------- | -------- | ------------------------------------------------------------------------------------------ |
| `action`   | `enum`   | yes      | Must be "generate_pdf".                                                                    |
| `vote_id`  | `string` | yes      | The selected vote.                                                                         |
| `timezone` | `string` | no       | Timezone used in the protocol, defaults to UTC, IANA format, e.g."CET" or "Europe/Vienna". |

##### Example

```json
{
    "action": "generate_pdf",
    "vote_id": "00000000-0000-0000-0000-000000000000",
    "timezone": "CET"
}
```

#### Response

When the PDF got created, a [PdfAsset](#pdfasset) response is sent to the issuer.

---

## Events

Events are received by participants when the vote state is changed. Events may be a 'direct' response to an issued command or unrelated
to the actions of the receiving participant.

### Started

A vote has been started by a moderator.

This message will also be received when joining a room that has an active vote going.

#### Fields

| Field                  | Type       | Required | Description                                                                               |
| ---------------------- | ---------- | -------- | ----------------------------------------------------------------------------------------- |
| `message`              | `enum`     | yes      | Is `"started"`.                                                                           |
| `kind`                 | `enum`     | yes      | The exhaustive list of `kind` can be found in section [Vote Kind](#vote-kind).            |
| `initiator_id`         | `string`   | yes      | Id of the participant which started the vote.                                             |
| `legal_vote_id`        | `string`   | yes      | Id of the vote.                                                                           |
| `start_time`           | `string`   | yes      | RFC 3339 timestamp when the vote started.                                                 |
| `max_votes`            | `int`      | yes      | The maximum number of possible votes.                                                     |
| `name`                 | `string`   | yes      | General name of the vote.                                                                 |
| `subtitle`             | `string`   | no       | A subtitle for the vote                                                                   |
| `topic`                | `string`   | no       | Detailed topic that will be voted on.                                                     |
| `allowed_participants` | `string[]` | yes      | An array of participant ids, where each contained participant is allowed to cast a vote.  |
| `enable_abstain`       | `bool`     | yes      | Enable/Disable the 'Abstain' option on this vote.                                         |
| `auto_close`           | `bool`     | yes      | When set, the vote will automatically close when every allowed participant casted a vote. |
| `create_pdf`           | `bool`     | yes      | Automatically create a protocol PDF when the vote ends.                                   |
| `duration`             | `int`      | no       | Duration of the vote in seconds, counting from the `start_time`.                          |
| `token`                | `string`   | no       | Optional. Only users who participate in the voting procedure receive a token.             |

##### Example

```json
{
    "message": "started",
    "kind": "roll_call",
    "initiator_id": "00000000-0000-0000-0000-000000000004",
    "legal_vote_id": "00000000-0000-0000-0000-000000000123",
    "start_time": "1970-01-01T00:00:00Z",
    "max_votes": 3,
    "name": "Vote Test",
    "topic": "Yes or No?",
    "allowed_participants": [
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000002",
        "00000000-0000-0000-0000-000000000003"
    ],
    "enable_abstain": false,
    "auto_close": false,
    "duration": 60,
    "token": "2QNav7b3FJw"
}
```

---

### Voted

Event received by a user whenever they voted. Usually understood as a response to the [Vote](#vote) command.
Since every user may only vote once, each participant logged in as that user will receive this message after
any of them successfully cast their vote.

#### Fields

| Field            | Type     | Required                     | Description                                      |
| ---------------- | -------- | ---------------------------- | ------------------------------------------------ |
| `message`        | `enum`   | yes                          | Is `"voted"`.                                    |
| `legal_vote_id`  | `string` | yes                          | Id of the vote.                                  |
| `response`       | `enum`   | yes                          | Either `"success"` or `"failed"`                 |
| `vote_option`    | `enum`   | when `response` is `success` | The option the `issuer` voted for                |
| `issuer`         | `string` | when `response` is `success` | Id of the participant which voted.               |
| `consumed_token` | `string` | when `response` is `success` | The token that is consumed once a user has voted |
| `reason`         | `enum`   | when `response` is `failed`  | Reason why the vote failed, see table below      |

__Failure reason:__

| Reason            | Description                                                  |
| ----------------- | ------------------------------------------------------------ |
| `invalid_vote_id` | the field `legal_vote_id` contained an invalid or unknown id |
| `ineligible`      | the user is not eligible to vote                             |
| `invalid_option`  | the given `vote_option` field contained an unknown option    |

##### Examples

Successful vote:

```json
{
    "message": "voted",
    "legal_vote_id": "00000000-0000-0000-0000-000000000123",
    "response": "success",
    "vote_option": "yes",
    "issuer": "00000000-0000-0000-0000-000000000001",
    "consumed_token": "2QNav7b3FJw"
}
```

Failed vote:

```json
{
    "message": "voted",
    "legal_vote_id": "00000000-0000-0000-0000-000000000123",
    "response": "failed",
    "reason": "ineligible"
}
```

---

### Updated

Update to an ongoing vote which supports live updates, signaling the newest results. Used to visualize the UI.

#### Fields

| Field           | Type     | Required                        | Description                                             |
| --------------- | -------- | ------------------------------- | ------------------------------------------------------- |
| `message`       | `enum`   | yes                             | Is `"updated"`.                                         |
| `legal_vote_id` | `string` | yes                             | Id of the vote.                                         |
| `yes`           | `int`    | yes                             | Number of `"yes"` votes                                 |
| `no`            | `int`    | yes                             | Number of `"no"` votes                                  |
| `abstain`       | `int`    | when `enable_abstain` is `true` | Number of `"abstain"` votes                             |
| `voting_record` | `map`    | yes                             | Mapping of participant which voted to their vote option |

##### Example

```json
{
    "message": "updated",
    "legal_vote_id": "00000000-0000-0000-0000-000000000123",
    "yes": 1,
    "no": 2,
    "abstain": 2,
    "voting_record": {
        "00000000-0000-0000-0000-000000000001": "yes",
        "00000000-0000-0000-0000-000000000002": "no",
        "00000000-0000-0000-0000-000000000003": "abstain",
        "00000000-0000-0000-0000-000000000004": "abstain",
        "00000000-0000-0000-0000-000000000005": "no"
    }
}
```

---

### Stopped

An ongoing vote has been finished and the results are being distributed with this event. The vote results may be
invalid, this happens when the final vote results are not consistent or altered unexpectedly.

#### Fields

| Field           | Type     | Required                        | Description                                                                                               |
| --------------- | -------- | ------------------------------- | --------------------------------------------------------------------------------------------------------- |
| `message`       | `enum`   | yes                             | Is `"stopped"`.                                                                                           |
| `legal_vote_id` | `string` | yes                             | Id of the vote.                                                                                           |
| `kind`          | `enum`   | yes                             | Either `"by_participant"`, `"auto"` or `"expired"`                                                        |
| `issuer`        | `string` | when `kind` is `by_participant` | Id of the participant which issued the stop command                                                       |
| `results`       | `enum`   | yes                             | Is either `valid` or `invalid`. This field changes the rest of the fields to one of the following tables. |
| `end_time`      | `string` | yes                             | RFC 3339 timestamp when the vote ended.                                                                   |

When `results` is `valid`:

| Field           | Type  | Required                        | Description                                                 |
| --------------- | ----- | ------------------------------- | ----------------------------------------------------------- |
| `yes`           | `int` | yes                             | Number of `"yes"` votes                                     |
| `no`            | `int` | yes                             | Number of `"no"` votes                                      |
| `abstain`       | `int` | when `enable_abstain` is `true` | Number of `"abstain"` votes                                 |
| `voting_record` | `map` | yes                             | Mapping of participant or token including their vote option |

When `invalid`:

| Field    | Type   | Required | Description                                                |
| -------- | ------ | -------- | ---------------------------------------------------------- |
| `reason` | `enum` | yes      | Either `"abstain_disabled"` or `"vote_count_inconsistent"` |

##### Example

Stopped with valid results on a vote which reveals the users:

```json
{
    "message": "stopped",
    "legal_vote_id": "00000000-0000-0000-0000-000000000123",
    "kind": "by_participant",
    "issuer":  "00000000-0000-0000-0000-000000000001",
    "results": "valid",
    "yes": 1,
    "no": 2,
    "abstain": 2,
    "voting_record": {
        "00000000-0000-0000-0000-000000000001": "yes",
        "00000000-0000-0000-0000-000000000002": "no",
        "00000000-0000-0000-0000-000000000003": "abstain",
        "00000000-0000-0000-0000-000000000004": "abstain",
        "00000000-0000-0000-0000-000000000005": "no"
    },
    "end_time": "1970-01-01T00:00:00Z"
}
```

Stopped with valid results on a vote which reveals the tokens:

```json
{
    "message": "stopped",
    "legal_vote_id": "00000000-0000-0000-0000-000000000123",
    "kind": "by_participant",
    "issuer":  "00000000-0000-0000-0000-000000000001",
    "results": "valid",
    "yes": 1,
    "no": 2,
    "abstain": 2,
    "voting_record": {
        "9AMndyeorvB": "yes",
        "G9rLx7vkeMD": "no",
        "Mypgay5rhRj": "abstain",
        "TjR94viayBf": "abstain",
        "UuLLU1sxgPw": "no"
    },
    "end_time": "1970-01-01T00:00:00Z"
}
```

Stopped with invalid results:

```json
{
    "message": "stopped",
    "legal_vote_id": "00000000-0000-0000-0000-000000000123",
    "kind": "by_participant",
    "issuer":  "00000000-0000-0000-0000-000000000001",
    "results": "invalid",
    "reason": "vote_count_inconsistent",
    "end_time": "1970-01-01T00:00:00Z"
}
```

---

### Canceled

An ongoing vote with `legal_vote_id` has been canceled. This may either be issued by a moderator or by the server when some
error or inconsistency occurred.

#### Fields

| Field           | Type     | Required                  | Description                                                 |
| --------------- | -------- | ------------------------- | ----------------------------------------------------------- |
| `message`       | `enum`   | yes                       | Is `"canceled"`.                                            |
| `legal_vote_id` | `string` | yes                       | Id of the vote.                                             |
| `reason`        | `enum`   | yes                       | Either `"custom"`, `"room_destroyed"` or `"initiator_left"` |
| `custom`        | `string` | when `reason` is `custom` | The reason for the cancel, set by the moderator.            |

##### Examples

```json
{
    "message": "canceled",
    "legal_vote_id": "00000000-0000-0000-0000-000000000000",
    "reason": "initiator_left"
}
```

With custom reason:

```json
{
    "message": "canceled",
    "legal_vote_id": "00000000-0000-0000-0000-000000000000",
    "reason": "custom",
    "custom": "Some important voters left"
}
```

---

### ReportedIssue

Received by the vote creator when a participant reports an issue via the [`ReportIssue`](#reportissue) action.

The reported issues are saved and displayed in the vote protocol.

#### Fields

| Field            | Type     | Required | Description                                                                                  |
| ---------------- | -------- | -------- | -------------------------------------------------------------------------------------------- |
| `message`        | `enum`   | yes      | Is `"reported_issue"`.                                                                       |
| `legal_vote_id`  | `string` | yes      | The ID of the related legal vote                                                             |
| `participant_id` | `string` | no       | The participant ID of the user that issued the report. Omitted when the vote is pseudonymous |
| `kind`           | `enum`   | no       | Either `"audio"`, `"video"` or `"screenshare"`.                                              |
| `description`    | `string` | no       | An optional message to the vote creator. Is mandatory when no `kind` is provided             |

#### Example

```json
{
    "message": "reported_issue",
    "participant_id": "00000000-0000-0000-0000-000000000000",
    "kind": "video",
    "description": "Hello, my video is not working"
}
```

```json
{
    "message": "reported_issue",
    "participant_id": "00000000-0000-0000-0000-000000000000",
    "description": "Something else is not working"
}
```

---

### PdfAsset

A PDF document has been created for the specified vote.

This message is sent to the issuing moderator when the vote ends and `create_pdf` field in the [Start](#start) message
is set. In case of an automatic stop, the message is sent to the vote initiator.

#### Fields

| Field           | Type     | Required | Description              |
| --------------- | -------- | -------- | ------------------------ |
| `message`       | `enum`   | yes      | Is "pdf_asset".          |
| `legal_vote_id` | `string` | yes      | Id of the vote.          |
| `asset_id`      | `string` | yes      | Id of the created asset. |

##### Examples

```json
{
    "message": "pdf_asset",
    "legal_vote_id": "00000000-0000-0000-0000-000000000000",
    "asset_id": "00000000-0000-0000-0000-000000000000"
}
```

---

### Error

The error event is a message that may be triggered by syntactically correct but invalid commands inside the `legal-vote` namespace
and therefore could be considered a kind of response. Errors must be handled outside of any context as they are considered events
that can happen at any time. (e.g. an `internal` error may occur at any time to signal an internal problem)

| Field     | Type       | Required                                      | Description                                          |
| --------- | ---------- | --------------------------------------------- | ---------------------------------------------------- |
| `message` | `enum`     | yes                                           | Is `"error"`.                                        |
| `error`   | `enum`     | yes                                           | Exhaustive list of error strings, see table below    |
| `guests`  | `string[]` | when `error` is `"allowlist_contains_guests"` | A list of participants that where found to be quests |

| Error                       | Description                                                                                     |
| --------------------------- | ----------------------------------------------------------------------------------------------- |
| `vote_already_active`       | Start command sent while a vote was already active                                              |
| `no_vote_active`            | A vote related related request was sent while no vote is active                                 |
| `invalid_vote_id`           | An invalid vote id was references inside a command                                              |
| `ineligible`                | The requesting user is ineligible for the issued command                                        |
| `allowlist_contains_guests` | The `allow_list` of a [Start](#start) provided `start` message contained guests                 |
| `permission_error`          | Failed to set permissions when creating backend resources                                       |
| `insufficient_permissions`  | The requesting user has insufficient permissions (E.g. a command requires the moderator role    |
| `internal`                  | Backend services encountered an internal error and any active vote should be considered invalid |

#### Example

```json
{
    "message": "error",
    "error": "vote_already_active"
}
```

Error for an invalid [Start](#start) request, where the allowlist contains guests:

```json
{
    "message": "error",
    "error": "allowlist_contains_guest",
    "guests": ["00000000-0000-0000-0000-000000000123", "00000000-0000-0000-0000-000000011311"]
}
```
