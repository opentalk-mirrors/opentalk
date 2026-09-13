// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Contains information about the recurrence of a new event.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventRecurrence {
    /// The recurrence pattern for recurring events.
    pub recurrence_pattern: String,
}
