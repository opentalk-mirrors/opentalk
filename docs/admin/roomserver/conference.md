# Conference

## Configuration

The section in the [configuration file](configuration.md) is called `conference`.

| Field               | Type     | Required | Default value                          | Description                                                                         |
| ------------------- | -------- | -------- | -------------------------------------- | ----------------------------------------------------------------------------------- |
| `signaling_salt`    | `string` | no       | random generated 24 character `string` | A random salt is generated at startup when not set. This value must be kept secret. |
| `room_idle_timeout` | `u64`    | no       | `60`                                   | The duration in seconds after which a room without participants is closed.          |

## Signaling Salt

!!! danger

    This value must be kept secret.

The signaling salt is used to derive a participants device id from their device secret.
The device id is used as a unique identifier for a device.
When not set, a random 24 character string is generated at startup.
This means, that the signaling salt changes on each restart when not configured.

## Room Idle Timeout

When all participants leave a room, it is not destroyed immediately.
This is to ensure that the state of the room is not lost, when no participant is present for a short time.
The room idle timeout determines how many seconds the RoomServer waits until destroying a room and dropping its state.
