// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The reason for the Control::Left event
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum LeaveReason {
    /// The participant quit
    Quit,
    /// The websocket connection timed out
    Timeout,
    /// Sent to waiting room
    SentToWaitingRoom,
    /// The participant was kicked by a moderator
    Kicked,
    /// The participant was banned by a moderator
    Banned,
}
