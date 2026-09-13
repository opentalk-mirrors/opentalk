// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::time::Duration;

/// Representation of an update to a training participation report parameter set in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEventTrainingParticipationReportParameterSet {
    /// The minimum duration until the initial checkpoint.
    pub initial_checkpoint_delay_after: Option<Duration>,

    /// The timespan within which the initial checkpoint happens.
    pub initial_checkpoint_delay_within: Option<Duration>,

    /// The minimum duration for the checkpoint interval.
    pub checkpoint_interval_after: Option<Duration>,

    /// The timespan within which the checkpoint interval happens.
    pub checkpoint_interval_within: Option<Duration>,
}
