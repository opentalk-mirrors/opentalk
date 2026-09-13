// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::prelude::*;
use opentalk_inventory as inventory;

use crate::{
    schema::event_training_participation_report_parameter_sets,
    tables::event_training_participation_report_parameter_sets::Duration,
};

#[derive(AsChangeset)]
#[diesel(table_name = event_training_participation_report_parameter_sets)]
pub struct UpdateEventTrainingParticipationReportParameterSet {
    pub initial_checkpoint_delay_after: Option<Duration>,
    pub initial_checkpoint_delay_within: Option<Duration>,
    pub checkpoint_interval_after: Option<Duration>,
    pub checkpoint_interval_within: Option<Duration>,
}

impl From<inventory::UpdateEventTrainingParticipationReportParameterSet>
    for UpdateEventTrainingParticipationReportParameterSet
{
    fn from(
        inventory::UpdateEventTrainingParticipationReportParameterSet {
            initial_checkpoint_delay_after,
            initial_checkpoint_delay_within,
            checkpoint_interval_after,
            checkpoint_interval_within,
        }: inventory::UpdateEventTrainingParticipationReportParameterSet,
    ) -> Self {
        Self {
            initial_checkpoint_delay_after: initial_checkpoint_delay_after.map(Into::into),
            initial_checkpoint_delay_within: initial_checkpoint_delay_within.map(Into::into),
            checkpoint_interval_after: checkpoint_interval_after.map(Into::into),
            checkpoint_interval_within: checkpoint_interval_within.map(Into::into),
        }
    }
}
