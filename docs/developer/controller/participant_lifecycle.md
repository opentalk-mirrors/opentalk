# Participant Lifecycle and States

This document provides a technical overview of the participant lifecycle,
complementing the [admin documentation](https://docs.opentalk.eu/admin/controller/under_the_hood/participant_states/).

The state of a participant is influenced by several factors. Primarily, whether
a user or guest can access the meeting depends on their invitation status. When
invited, a guest receives a guest link containing a secret required to join the
meeting. Registered users with an account can join meetings if personally invited
or if they have received a guest link. The invitation status is stored in the
database.

## Joining the meeting

Once a user or guest joins a meeting, their participant state is tracked either
in memory or Redis, depending on the controller's configuration. For each
participant, a runner instance is started, and the state is managed through
the `RunnerState`.

```rust
/// Current state of the runner
#[derive(Debug, Clone, PartialEq, Eq)]
enum RunnerState {
    /// Runner and its message exchange resources are created
    /// but has not joined the room yet (no redis resources set)
    None,

    /// Inside the waiting room
    Waiting {
        accepted: bool,
        control_data: ControlState,
    },

    /// Inside the actual room
    Joined,
}
```

## Moderator actions

Moderators have the authority to remove participants from a meeting, with varying
consequences. A participant can either be sent to the waiting room—where the
moderator can later admit them—or be permanently removed from the meeting. If
permanently removed, the participant may be banned from rejoining, or they may
still be allowed to re-enter the meeting.

### ModerationCommand::Kick

When a participant is kicked from the meeting, they are removed without being
banned, and can rejoin at any time. If the meeting is part of a recurring event,
they can also join future sessions of the event.

If the waiting room feature is enabled, the participant will be placed in the
waiting room upon rejoining. A moderator must then admit them back into the
meeting.

### ModerationCommand::Ban

!!! info Guests cannot be banned

    Since guest have no persistent participant id or user id, they cannot be banned only kicked.

When a participant is banned from a meeting, the same actions as a kick are applied.
However, an entry is also added to volatile storage, marking the user ID as banned.
This prevents the user from rejoining until the meeting ends and the volatile storage
is cleared. For recurring meetings, the participant can join future sessions after
the current one ends.

### ModerationCommand::SendToWaitingRoom

When a participant is sent to the waiting room, they are not fully removed from the meeting. Instead, their `RunnerState` is reset to `Waiting`, and their module states are cleared. If the waiting room was not previously enabled, it is activated by this command.
