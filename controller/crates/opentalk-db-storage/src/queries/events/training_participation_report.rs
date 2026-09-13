// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::events::EventId;

use crate::{
    schema::event_training_participation_report_parameter_sets,
    tables::event_training_participation_report_parameter_sets::{
        EventTrainingParticipationReportParameterSet,
        UpdateEventTrainingParticipationReportParameterSet,
    },
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_training_participation_report_parameter_set(
    conn: &mut DbConnection,
    event_id: EventId,
) -> Result<Option<EventTrainingParticipationReportParameterSet>> {
    event_training_participation_report_parameter_sets::table
        .filter(event_training_participation_report_parameter_sets::event_id.eq(event_id))
        .get_result(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

/// Tries to insert the EventTrainingParticipationParameterSet into the database
///
/// When yielding a unique key violation, None is returned.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn try_create_event_training_participation_report_parameter_set(
    conn: &mut DbConnection,
    new_event_training_participation_report_parameter_set: EventTrainingParticipationReportParameterSet,
) -> Result<Option<EventTrainingParticipationReportParameterSet>> {
    let result = diesel::insert_into(event_training_participation_report_parameter_sets::table)
        .values(new_event_training_participation_report_parameter_set)
        .get_result(conn)
        .await;

    match result {
        Ok(event_invite) => Ok(Some(event_invite)),
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            ..,
        )) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_event_training_participation_report_set(
    conn: &mut DbConnection,
    event_id: EventId,
) -> Result<()> {
    diesel::delete(
        event_training_participation_report_parameter_sets::table
            .filter(event_training_participation_report_parameter_sets::event_id.eq(event_id)),
    )
    .execute(conn)
    .await?;

    Ok(())
}

/// Apply the update to the invite where `user_id` is the invitee.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_event_training_participation_report_parameter_set(
    conn: &mut DbConnection,
    event_id: EventId,
    parameter_set: UpdateEventTrainingParticipationReportParameterSet,
) -> Result<EventTrainingParticipationReportParameterSet> {
    diesel::update(event_training_participation_report_parameter_sets::table)
        .filter(event_training_participation_report_parameter_sets::event_id.eq(event_id))
        .set(parameter_set)
        // change it here
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}
