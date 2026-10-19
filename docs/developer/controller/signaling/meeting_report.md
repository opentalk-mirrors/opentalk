# Meeting Reports

The Meeting Reports module allows users to generate attendance reports. An
attendance report contains a list of participants including the time they
joined and left.

## Joining the room

### JoinSuccess

When joining a room, the `join_success` control event does not contain
module-specific data.

### Joined

When joining a room, the `joined` control event sent to all other participants
does not contain module-specific data.

## Commands

### GenerateMeetingReport

Allows a moderator to generate a meeting report. The meeting report will be stored as an asset.

#### Fields

| Field                     | Type   | Required | Description                                                                   |
| ------------------------- | ------ | -------- | ----------------------------------------------------------------------------- |
| `action`                  | `enum` | yes      | Must be `"generate_attendance_report"`                                        |
| `include_email_addresses` | `bool` | yes      | `true` if email addresses should be included in the report, `false` otherwise |

#### Example

```json
{
    "action": "generate_attendance_report",
    "include_email_addresses": false
}
```

---

## Events

### PdfAsset

A meeting report has been generated and stored with the asset ID and file name.

#### Fields

| Field      | Type      | Always | Description                               |
| ---------- | --------- | ------ | ----------------------------------------- |
| `message`  | `enum`    | yes    | Is `"pdf_asset"`                          |
| `filename` | `string`  | yes    | File name of the generated meeting report |
| `asset_id` | `AssetId` | yes    | Id of the asset inside the object storage |

#### Example

```json
{
    "message": "pdf_asset",
    "filename": "global",
    "asset_id": "00000000-0000-0000-0000-000000000000"
}
```

### Error

Received when something went wrong generating a meeting report on the server.

#### Fields

| Field     | Type   | Always | Description                                       |
| --------- | ------ | ------ | ------------------------------------------------- |
| `message` | `enum` | yes    | Is `"error"`                                      |
| `error`   | `enum` | yes    | Exhaustive list of error strings, see table below |

| Error                      | Description                                                        |
| -------------------------- | ------------------------------------------------------------------ |
| `insufficient_permissions` | The requesting user has insufficient permissions for the operation |
| `storage_exceeded`         | The requesting user has exceeded their storage                     |
| `generate_failed`          | Internal error while generating the report                         |

#### Example

```json
{
    "message": "error",
    "error": "generate_failed"
}
```
