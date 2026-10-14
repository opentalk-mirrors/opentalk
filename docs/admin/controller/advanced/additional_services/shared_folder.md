# Shared Folders on External Storage Systems

This feature allows to create shared folders on external storage systems. The
following systems are supported:

- [NextCloud](https://nextcloud.com/)

## Functionality

When the shared folder is enabled, a folder per meeting room can be created
when the meeting is configured, and shared with other participants during the
meeting.

### Creating a shared folder for a meeting

A shared folder can be created for an existing meeting room. The procedure is:

- A folder is created on the configured external storage system.
- A read-write share for the folder is created with a random password. The
  corresponding link and password will be available to participants with
  moderation permission only.
- A read-only share for the folder is created with a random password. The
  corresponding link and password will be available to all participants.

### Deleting a shared folder from a meeting

When deleting a shared folder from a meeting, this procedure is performed:

- The read-only and the read-write shares are deleted.
- The folder is deleted recursively, including all files that it contains.

### Accessing the shared folder during a meeting

If a folder exists for a meeting at the moment when it is started, the signaling messages
will contain the information required by a client to show the link and the password.

A participant with moderation permissions will see the link and the password for
the read-write share, whereas a participant without moderation permissions will
see the link and the password for the read-only share.

If a participant is assigned moderation permission during the meeting, they will
receive an update message with the read-write link and password. When the moderation
permission gets revoked, an update message with the read-only link is sent to the
client.

## NextCloud

A NextCloud instance can be configured through the [configuration file](../../core/configuration.md).

### Configuration fields

The section in the [configuration file](../../core/configuration.md) is called `shared_folder`, and it knows these fields:

| Field       | Type     | Required | Default value | Description                                                                                                             |
| ----------- | -------- | -------- | ------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `provider`  | `string` | yes      | -             | Must be `"nextcloud"` when using the NextCloud provider.                                                                |
| `url`       | `string` | yes      | -             | The complete URL of the NextCloud installation.                                                                         |
| `username`  | `string` | yes      | -             | Username of the account that is used for logging into the NextCloud service.                                            |
| `password`  | `string` | yes      | -             | Password of the account.                                                                                                |
| `directory` | `string` | no       | `""`          | Path to the directory on the server where shared folders will be created.                                               |
| `expiry`    | `uint`   | no       | not set       | If set, the shares will be created with an expiry of that value (number of days), otherwise the shares will not expire. |

### Example configuration

Example section inside the configuration file with the shared folder configured for NextCloud:

```toml
[shared_folder]
provider = "nextcloud"
url = "https://nextcloud.example.org/"
username = "exampleuser"
password = "v3rys3cr3t"
directory = "opentalk/meetings"
expiry = 48
```

## Developer information

- [REST API documentation](https://opentalk.eu/docs/developer/controller/rest/#tag/shared_folder)
- [Signaling API](https://opentalk.eu/docs/developer/controller/signaling/community/shared_folder)
