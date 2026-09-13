// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_inventory::{
    EventTrainingParticipationReportInventory, EventTrainingParticipationReportParameterSet,
    UpdateEventTrainingParticipationReportParameterSet,
};
use opentalk_types_common::events::EventId;
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl EventTrainingParticipationReportInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_training_participation_report_parameter_set(
        &mut self,
        event_id: EventId,
    ) -> Result<Option<EventTrainingParticipationReportParameterSet>> {
        Ok(
            db::queries::events::get_event_training_participation_report_parameter_set(
                &mut self.inner,
                event_id,
            )
            .await
            .context(DatabaseSnafu)?
            .map(Into::into),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_training_participation_report_parameter_set(
        &mut self,
        event_id: EventId,
        parameter_set: UpdateEventTrainingParticipationReportParameterSet,
    ) -> Result<EventTrainingParticipationReportParameterSet> {
        Ok(
            db::queries::events::update_event_training_participation_report_parameter_set(
                &mut self.inner,
                event_id,
                parameter_set.into(),
            )
            .await
            .context(DatabaseSnafu)?
            .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn try_create_event_training_participation_report_parameter_set(
        &mut self,
        parameter_set: EventTrainingParticipationReportParameterSet,
    ) -> Result<Option<EventTrainingParticipationReportParameterSet>> {
        Ok(
            db::queries::events::try_create_event_training_participation_report_parameter_set(
                &mut self.inner,
                parameter_set.into(),
            )
            .await
            .context(DatabaseSnafu)?
            .map(Into::into),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_event_training_participation_report_parameter_set(
        &mut self,
        event_id: EventId,
    ) -> Result<()> {
        Ok(
            db::queries::events::delete_event_training_participation_report_set(
                &mut self.inner,
                event_id,
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }
}
