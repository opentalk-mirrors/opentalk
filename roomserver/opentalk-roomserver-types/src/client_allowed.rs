// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

/// The result of checking whether a client is allowed to join a room.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClientAllowed {
    /// The client is allowed to join the room.
    Yes,
    /// The client is not allowed to join the room.
    No { reason: ClientNotAllowedReason },
}

/// The reason why a client is not allowed to join a room.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ClientNotAllowedReason {
    /// The client is banned from the room.
    Banned,
    /// The client is a guest and guest access is disabled for the room.
    GuestAccessDisabled,
    /// The client does not support e2ee (e.g. a service) and e2ee is enabled for the room.
    E2eeEnabled,
}
