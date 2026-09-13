// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::EventId,
    training_participation_report::{TimeRange, TrainingParticipationReportParameterSet},
};

/// The representation of a parameter set for the training participation report in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventTrainingParticipationReportParameterSet {
    /// The id of the event.
    pub event_id: EventId,

    /// The time range to use for determining the inital checkpoint.
    pub initial_checkpoint_delay: TimeRange,

    /// The time range to use for determining subsequent checkpoint intervals.
    pub checkpoint_interval: TimeRange,
}

impl From<EventTrainingParticipationReportParameterSet>
    for TrainingParticipationReportParameterSet
{
    fn from(
        EventTrainingParticipationReportParameterSet {
            event_id: _,
            initial_checkpoint_delay,
            checkpoint_interval,
        }: EventTrainingParticipationReportParameterSet,
    ) -> Self {
        Self {
            initial_checkpoint_delay,
            checkpoint_interval,
        }
    }
}

impl From<(EventId, TrainingParticipationReportParameterSet)>
    for EventTrainingParticipationReportParameterSet
{
    fn from(
        (
            event_id,
            TrainingParticipationReportParameterSet {
                initial_checkpoint_delay,
                checkpoint_interval,
            },
        ): (EventId, TrainingParticipationReportParameterSet),
    ) -> Self {
        Self {
            event_id,
            initial_checkpoint_delay,
            checkpoint_interval,
        }
    }
}
