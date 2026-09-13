// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The kinds of event exceptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventExceptionKind {
    /// The event instance is modified.
    Modified,

    /// The event instance is cancelled.
    Cancelled,
}
