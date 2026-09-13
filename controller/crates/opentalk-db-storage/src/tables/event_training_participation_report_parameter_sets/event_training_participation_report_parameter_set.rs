// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{Insertable, Queryable};
use opentalk_inventory as inventory;
use opentalk_types_common::{events::EventId, training_participation_report::TimeRange};

use crate::{
    schema::event_training_participation_report_parameter_sets,
    tables::{event_training_participation_report_parameter_sets::Duration, events::Event},
};

#[derive(Debug, Insertable, Queryable, Identifiable, Associations)]
#[diesel(table_name = event_training_participation_report_parameter_sets)]
#[diesel(primary_key(event_id))]
#[diesel(belongs_to(Event, foreign_key = event_id))]
pub struct EventTrainingParticipationReportParameterSet {
    pub event_id: EventId,
    pub initial_checkpoint_delay_after: Duration,
    pub initial_checkpoint_delay_within: Duration,
    pub checkpoint_interval_after: Duration,
    pub checkpoint_interval_within: Duration,
}

impl From<EventTrainingParticipationReportParameterSet>
    for inventory::EventTrainingParticipationReportParameterSet
{
    fn from(
        EventTrainingParticipationReportParameterSet {
            event_id,
            initial_checkpoint_delay_after,
            initial_checkpoint_delay_within,
            checkpoint_interval_after,
            checkpoint_interval_within,
        }: EventTrainingParticipationReportParameterSet,
    ) -> Self {
        Self {
            event_id,
            initial_checkpoint_delay: TimeRange::new_with_clamped_durations(
                initial_checkpoint_delay_after.into(),
                initial_checkpoint_delay_within.into(),
            ),
            checkpoint_interval: TimeRange::new_with_clamped_durations(
                checkpoint_interval_after.into(),
                checkpoint_interval_within.into(),
            ),
        }
    }
}

impl From<inventory::EventTrainingParticipationReportParameterSet>
    for EventTrainingParticipationReportParameterSet
{
    fn from(
        inventory::EventTrainingParticipationReportParameterSet {
            event_id,
            initial_checkpoint_delay,
            checkpoint_interval,
        }: inventory::EventTrainingParticipationReportParameterSet,
    ) -> Self {
        Self {
            event_id,
            initial_checkpoint_delay_after: initial_checkpoint_delay.after().into(),
            initial_checkpoint_delay_within: initial_checkpoint_delay.within().into(),
            checkpoint_interval_after: checkpoint_interval.after().into(),
            checkpoint_interval_within: checkpoint_interval.within().into(),
        }
    }
}
