// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::events::EventId;

use crate::{
    schema::event_exceptions,
    tables::event_exceptions::{
        EventException, EventExceptionId, NewEventException, UpdateEventException,
    },
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_exception(
    conn: &mut DbConnection,
    event_id: EventId,
    datetime: DateTime<Utc>,
) -> Result<Option<EventException>> {
    let result = event_exceptions::table
        .filter(
            event_exceptions::event_id
                .eq(event_id)
                .and(event_exceptions::exception_date.eq(datetime)),
        )
        .first(conn)
        .await
        .optional()?;

    Ok(result)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_exceptions(
    conn: &mut DbConnection,
    event_id: EventId,
    datetimes: &[&DateTime<Utc>],
) -> Result<Vec<EventException>> {
    let result = event_exceptions::table
        .filter(
            event_exceptions::event_id
                .eq(event_id)
                .and(event_exceptions::exception_date.eq_any(datetimes)),
        )
        .load(conn)
        .await
        .optional()?
        .unwrap_or_default();

    Ok(result)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_event_exceptions_for_event(
    conn: &mut DbConnection,
    event_id: EventId,
) -> Result<()> {
    diesel::delete(event_exceptions::table)
        .filter(event_exceptions::event_id.eq(event_id))
        .execute(conn)
        .await?;

    Ok(())
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_event_exception(
    conn: &mut DbConnection,
    update_event_exception: UpdateEventException,
    event_exception_id: EventExceptionId,
) -> Result<EventException> {
    diesel::update(event_exceptions::table)
        .filter(event_exceptions::id.eq(event_exception_id))
        .set(update_event_exception)
        .returning(event_exceptions::all_columns)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn create_event_exception(
    conn: &mut DbConnection,
    new_event_exception: NewEventException,
) -> Result<EventException> {
    diesel::insert_into(event_exceptions::table)
        .values(new_event_exception)
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}
