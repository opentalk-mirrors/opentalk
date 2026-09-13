# Meeting Notes

The Meeting Notes module allows participants to to collaboratively edit a document in real-time by utilizing `Etherpad`.
[Etherpad](https://etherpad.org/) is an open source online editor that allows users to collaboratively edit a document in real-time.
The Etherpad is a separate process that runs along the controller. When in a room, participants can click an access link that
opens another browser tab where the Etherpad document (`pad`) can be accessed.

The Etherpad's state is lazily initialized for each room. To initialize and make the etherpad accessible, the moderator has to select
one or more `writers` from the present participants. Once selected, each participant will receive either a write- or read url,
depending on if they were picked as a `writer`.

Each participant has an individual etherpad session that gets set as cookie on the client by the `ep_auth_session` Etherpad plugin.
The plugin provides the additional endpoint `/auth_session` that takes the clients session id as a query parameter, sets the
session cookie on the clients browser, and the forwards the client to the actual `pad`.

A participants write access can be revoked with the `deselect_write` message. The participant will then receive a new read url.

## Joining the room

### JoinSuccess

The `join_success` control event contains the module-specific fields described below.

#### Fields

The `meeting_notes` module does not provide data in the `join_success` message.

### Joined

When joining a room, the `joined` control event sent to all other participants does not contain module-specific data.

---

## Commands

### SelectWriter

The `SelectWriter` message can be sent by a moderator to select a number of participants as writers.
The underlying etherpad state gets lazily initialized when this message is received by the controller.

Can return [Error](#error) of kind `insufficient_permissions`, `currently_initializing` `failed_initialization`.

#### Fields

| Field             | Type       | Required | Description                                                                  |
| ----------------- | ---------- | -------- | ---------------------------------------------------------------------------- |
| `participant_ids` | `string[]` | yes      | An array of participant ids that shall get write access to the etherpad pad. |

##### Example

```json
{
    "action": "select_writer",
    "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
}
```

#### Response

Each participant receives an access url. Depending if they were selected as a writer they will receive
either a [WriteUrl](#writeurl) or [ReadUrl](#readurl).

---

### DeselectWriter

The `DeselectWriter` message can be sent by a moderator to deselect a number of writers.

The deselected participants will have their write session invalidated and will receive a read access url.

#### Response

Each selected participant that has write access, receives a new read-only access url.

Can return [Error](#error) of kind `insufficient_permissions`, `currently_initializing`, `not_initialized` or `invalid_participant_selection`.

#### Fields

| Field             | Type       | Required | Description                                                                   |
| ----------------- | ---------- | -------- | ----------------------------------------------------------------------------- |
| `participant_ids` | `[string]` | yes      | An array of participant ids that shall lose write access to the etherpad pad. |

#### Examples

```json
{
    "action": "deselect_writer",
    "participant_ids": ["00000000-0000-0000-0000-000000000000", "00000000-0000-0000-0000-000000000001"]
}
```

### GeneratePdf

Allows a moderator to generate a PDF from the current contents of the meeting notes.

Access to the PDF is given to all participants in the room via the `PdfAsset` event.

#### Response

A [`PdfAsset`](#pdfasset) message with the asset id of the PDF document is sent to every participant in the room.

#### Fields

| Field    | Type   | Required | Description               |
| -------- | ------ | -------- | ------------------------- |
| `action` | `enum` | yes      | Must be `"generate_pdf"`. |

#### Example

```json
{
    "action": "generate_pdf",
}
```

## Events

### WriteUrl

Received by participants who got selected as a writer by a moderator. (See [SelectWriter](#selectwriter))

#### Fields

| Field | Type       | Required | Description                                    |
| ----- | ---------- | -------- | ---------------------------------------------- |
| `url` | `string[]` | yes      | A url to the etherpad that grants write access |

##### Example

```json
{
    "message":"write_url",
    "url":"http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group"
}
```

### ReadUrl

Received by all participants that did not get selected as a writer by a moderator. (See [SelectWriter](#selectwriter))

#### Fields

| Field | Type       | Required | Description                                       |
| ----- | ---------- | -------- | ------------------------------------------------- |
| `url` | `string[]` | yes      | A url to the etherpad that will set  write access |

##### Example

```json
{
    "message":"read_url",
    "url":"http://localhost/auth_session?sessionID=s.session&padName=meeting_notes&groupID=g.group"
}
```

### Control Update

The `update` message in the `control` namespace contains the current etherpad access level of the updated participant.

This information is only present when the receiver is a moderator.

An update message is issued when a participants etherpad access level changes.

#### Fields

| Field      | Type   | Required | Description                                 |
| ---------- | ------ | -------- | ------------------------------------------- |
| `readonly` | `bool` | yes      | The current access level of the participant |

#### Examples

```json
{
    "namespace": "control",
    "timestamp": "2022-06-14T17:18:52Z",
    "payload": {
        "message": "update",
        "id": "24500907-334b-47d4-b54a-00db40b9a613",
        ...
        "meeting_notes": {
            "readonly": false
        }
    }
}
```

### PdfAsset

Contains the filename and asset id of the PDF document of the meeting notes.

This event is received by every participant when a moderator generates a PDF document for the meeting notes.

#### Fields

| Field      | Type     | Required | Description                                            |
| ---------- | -------- | -------- | ------------------------------------------------------ |
| `message`  | `enum`   | yes      | Is `"pdf_asset"`.                                      |
| `filename` | `string` | yes      | The file name of the PDF document of the meeting notes |
| `asset_id` | `string` | yes      | The id of the PDF document asset                       |

#### Example

```json
{
    "message": "pdf_asset",
    "filename": "meeting_notes_2023-01-16_12:34:56-UTC.pdf",
    "asset_id": "5be191a8-3eb1-4c79-afe1-d4857dcf0e73"
}
```

### Error

Received when something went wrong processing messages sent to the server.

#### Fields

| Field     | Type   | Always | Description                                       |
| --------- | ------ | ------ | ------------------------------------------------- |
| `message` | `enum` | yes    | Is `"error"`                                      |
| `error`   | `enum` | yes    | Exhaustive list of error strings, see table below |

| Error                      | Description                                                                                          |
| -------------------------- | ---------------------------------------------------------------------------------------------------- |
| `insufficient_permissions` | The requesting user has insufficient permissions for the operation                                   |
| `currently_initializing`   | A moderator just send a [SelectWriter](#selectwriter) message and the etherpad is still initializing |
| `failed_initialization`    | The etherpad initialization failed                                                                   |
| `not_initialized`          | The etherpad is not yet initialized                                                                  |
| `storage_exceeded`         | The requesting user has exceeded their storage                                                       |

##### Example

```json
{
    "message":"error",
    "error":"insufficient_permissions"
}
```

```json
{
    "message":"error",
    "error":"currently_initializing"
}
```

```json
{
    "message":"error",
    "error":"failed_initialization"
}
```

```json
{
    "message":"error",
    "error":"not_initialized"
}
```
