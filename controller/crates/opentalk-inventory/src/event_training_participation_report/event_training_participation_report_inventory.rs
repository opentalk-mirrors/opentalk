// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::EventId, training_participation_report::TrainingParticipationReportParameterSet,
};

use super::{
    EventTrainingParticipationReportParameterSet,
    UpdateEventTrainingParticipationReportParameterSet,
};
use crate::Result;

/// A trait for retrieving and storing event shared folder entities.
#[async_trait::async_trait]
pub trait EventTrainingParticipationReportInventory {
    /// Get the training participation report parameter set for an event.
    async fn get_event_training_participation_report_parameter_set(
        &mut self,
        event_id: EventId,
    ) -> Result<Option<EventTrainingParticipationReportParameterSet>>;

    /// Update the training participation report parameter set for an event.
    async fn update_training_participation_report_parameter_set(
        &mut self,
        event_id: EventId,
        parameter_set: UpdateEventTrainingParticipationReportParameterSet,
    ) -> Result<EventTrainingParticipationReportParameterSet>;

    /// Create a new training participation report parameter set for an event.
    async fn try_create_event_training_participation_report_parameter_set(
        &mut self,
        parameter_set: EventTrainingParticipationReportParameterSet,
    ) -> Result<Option<EventTrainingParticipationReportParameterSet>>;

    /// Delete the event training participation report parameter set for an event.
    async fn delete_event_training_participation_report_parameter_set(
        &mut self,
        event_id: EventId,
    ) -> Result<()>;
}

impl From<TrainingParticipationReportParameterSet>
    for UpdateEventTrainingParticipationReportParameterSet
{
    fn from(
        TrainingParticipationReportParameterSet {
            initial_checkpoint_delay,
            checkpoint_interval,
        }: TrainingParticipationReportParameterSet,
    ) -> Self {
        Self {
            initial_checkpoint_delay_after: Some(initial_checkpoint_delay.after()),
            initial_checkpoint_delay_within: Some(initial_checkpoint_delay.within()),
            checkpoint_interval_after: Some(checkpoint_interval.after()),
            checkpoint_interval_within: Some(checkpoint_interval.within()),
        }
    }
}
