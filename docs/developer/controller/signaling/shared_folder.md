# Shared Folder

The shared folder module allows moderators to share the link to a shared
folder hosted on an external service.

Currently supported service types:

- NextCloud

## Joining the room

### JoinSuccess

When joining a room, the `join_success` control event contains the module-specific fields described below if a shared folder has been configured for the event associated with the meeting.

#### Fields

| Field        | Type                 | Always | Description                                                                                     |
| ------------ | -------------------- | ------ | ----------------------------------------------------------------------------------------------- |
| `read`       | `SharedFolderAccess` | yes    | Read access to the shared folder.                                                               |
| `read_write` | `SharedFolderAccess` | no     | Read and write access to the shared folder. Only present if the user has moderation permission. |

##### Example

Moderator is joining a room with a shared folder:

```json
{
    "read": {
        "url": "https://nextcloud.example.com/s/TArrLyC3K7c5Jbg",
        "password": "DLgoYrFEoy"
    },
    "read_write": {
        "url": "https://nextcloud.example.com/s/9x8x4P4nztD7XgC",
        "password": "ZA4AG3D9BD"
    }
}
```

Non-moderator participant is joining a room with a shared folder:

```json
{
    "read": {
        "url": "https://nextcloud.example.com/s/TArrLyC3K7c5Jbg",
        "password": "DLgoYrFEoy"
    }
}
```

### Joined

When joining a room, the `joined` control event sent to all other participants does not contain module-specific data.

---

## Commands

This module does not define any commands

---

## Events

Events are received by participants when the shared folder state has changed.

### Updated

Information about a shared folder has been updated, e.g. by getting moderation permissions granted or revoked.

#### Fields

| Field        | Type                  | Always | Description                                                                                             |
| ------------ | --------------------- | ------ | ------------------------------------------------------------------------------------------------------- |
| `message`    | `enum`                | yes    | Is `"updated"`                                                                                          |
| `read`       | `SharedFolderAccess`  | yes    | Read access to the shared folder.                                                                       |
| `read_write` | `SharedFolderAccess`  | no     | Read and write access to the shared folder. Only present if the current user has moderation permission. |

##### Example

`Updated` message received by a moderator:

```json
{
    "message": "updated",
    "read": {
        "url": "https://nextcloud.example.com/s/TArrLyC3K7c5Jbg",
        "password": "DLgoYrFEoy"
    },
    "read_write": {
        "url": "https://nextcloud.example.com/s/9x8x4P4nztD7XgC",
        "password": "ZA4AG3D9BD"
    }
}
```

`Updated` message received by a non-moderator:

```json
{
    "message": "updated",
    "read": {
        "url": "https://nextcloud.example.com/s/TArrLyC3K7c5Jbg",
        "password": "DLgoYrFEoy"
    }
}
```

---

## Shared Types

### SharedFolderAccess

The information required to access a shared folder.

#### Fields

| Field      | Type     | Always | Description                                                   |
| ---------- | -------- | ------ | ------------------------------------------------------------- |
| `url`      | `string` | yes    | The URL where the shared folder can be accessed.              |
| `password` | `string` | yes    | A password required for accessing the shared folder contents. |
