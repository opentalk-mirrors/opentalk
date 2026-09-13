// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains events database queries

pub mod types;

mod email_invite;
mod exception;
mod favorite;
mod invite;
mod shared_folder;
mod training_participation_report;

use chrono::{DateTime, Utc};
use diesel::{
    expression::AsExpression,
    pg::Pg,
    prelude::*,
    sql_types::{Nullable, Record, Timestamptz, Uuid},
};
use diesel_async::{AsyncConnection, RunQueryDsl};
pub use email_invite::*;
pub use exception::*;
pub use favorite::*;
use futures_core::Stream;
pub use invite::*;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    events::{
        EventId,
        invites::{EventInviteStatus, InviteRole},
    },
    rooms::{RoomId, RoomIdOrAlias},
    users::UserId,
};
pub use shared_folder::*;
pub use training_participation_report::*;

use crate::{
    queries::{
        events::types::{
            EventRecord, GetEventExceptionsCursor, GetEventsCursor, NewEventRecord,
            UpdateEventRecord,
        },
        room_filter::FilterByRoom as _,
    },
    schema::{
        event_dates, event_exceptions, event_favorites, event_invites, event_recurrences,
        event_shared_folders, event_training_participation_report_parameter_sets, events, rooms,
        sip_configs, tariffs, users,
    },
    tables::{
        event_exceptions::EventException, event_invites::EventInvite,
        event_shared_folders::EventSharedFolder,
        event_training_participation_report_parameter_sets::EventTrainingParticipationReportParameterSet,
        events::Event, rooms::Room, sip_configs::SipConfig, tariffs::Tariff, users::User,
    },
    utils::convert_diesel_query_results,
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event(conn: &mut DbConnection, event_id: EventId) -> Result<EventRecord> {
    events::table
        .left_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .left_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((
            events::all_columns,
            event_dates::all_columns.nullable(),
            event_recurrences::all_columns.nullable(),
        ))
        .filter(events::id.eq(event_id))
        .filter(users::disabled_since.is_null())
        .first(conn)
        .await
        .map_err(DatabaseError::from)
}

pub async fn get_all_event_and_room_ids_with_creator(
    conn: &mut DbConnection,
) -> Result<Vec<(EventId, UserId)>> {
    let events = events::table
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((events::id, events::created_by))
        .filter(users::disabled_since.is_null())
        .load(conn)
        .await?;

    Ok(events)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_event_and_room_ids_that_ended_before(
    conn: &mut DbConnection,
    date: DateTime<Utc>,
) -> Result<Vec<(EventId, RoomId)>> {
    events::table
        .inner_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .inner_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((events::id, events::room))
        .filter(event_dates::ends_at.le(date))
        .filter(users::disabled_since.is_null())
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_event_and_room_ids_adhoc_created_before(
    conn: &mut DbConnection,
    date: DateTime<Utc>,
) -> Result<Vec<(EventId, RoomId)>> {
    events::table
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((events::id, events::room))
        .filter(events::created_at.le(date))
        .filter(events::is_adhoc.eq(true))
        .filter(users::disabled_since.is_null())
        .load(conn)
        .await
        .map_err(Into::into)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_events_and_room_ids_for_creator_including_disabled(
    conn: &mut DbConnection,
    created_by: UserId,
) -> Result<Vec<(EventId, RoomId)>> {
    events::table
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((events::id, events::room))
        .filter(events::created_by.eq(created_by))
        .load(conn)
        .await
        .map_err(Into::into)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_events_finite_recurring(conn: &mut DbConnection) -> Result<Vec<EventRecord>> {
    events::table
        .inner_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .inner_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((
            events::all_columns,
            event_dates::all_columns.nullable(),
            event_recurrences::all_columns.nullable(),
        ))
        .filter(
            event_recurrences::recurrence_pattern
                .ilike("%UNTIL%")
                .or(event_recurrences::recurrence_pattern.ilike("%COUNT%")),
        )
        .filter(users::disabled_since.is_null())
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_events_updated_by_user(
    conn: &mut DbConnection,
    updated_by: UserId,
) -> Result<Vec<EventRecord>> {
    events::table
        .left_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .left_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((
            events::all_columns,
            event_dates::all_columns.nullable(),
            event_recurrences::all_columns.nullable(),
        ))
        .filter(events::updated_by.eq(updated_by))
        .filter(users::disabled_since.is_null())
        .load(conn)
        .await
        .map_err(Into::into)
}

pub async fn get_all_events_with_invitee(
    conn: &mut DbConnection,
) -> Result<Vec<(EventId, RoomId, UserId, InviteRole)>> {
    events::table
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .inner_join(event_invites::table.on(event_invites::event_id.eq(events::id)))
        .select((
            events::id,
            events::room,
            event_invites::invitee,
            event_invites::role,
        ))
        .filter(users::disabled_since.is_null())
        .load(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
#[expect(clippy::type_complexity)]
pub async fn get_with_related_items(
    conn: &mut DbConnection,
    user_id: UserId,
    event_id: EventId,
) -> Result<(
    EventRecord,
    Option<EventInvite>,
    Room,
    Option<SipConfig>,
    bool,
    Option<EventSharedFolder>,
    Tariff,
    Option<EventTrainingParticipationReportParameterSet>,
)> {
    events::table
        .left_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .left_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .left_join(
            event_invites::table.on(event_invites::event_id
                .eq(events::id)
                .and(event_invites::invitee.eq(user_id))),
        )
        .left_join(
            event_favorites::table.on(event_favorites::event_id
                .eq(events::id)
                .and(event_favorites::user_id.eq(user_id))),
        )
        .left_join(event_shared_folders::table.on(event_shared_folders::event_id.eq(events::id)))
        .inner_join(rooms::table.on(events::room.eq(rooms::id)))
        .left_join(sip_configs::table.on(rooms::id.eq(sip_configs::room)))
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .inner_join(tariffs::table.on(tariffs::id.eq(users::tariff_id)))
        .left_join(
            event_training_participation_report_parameter_sets::table
                .on(event_training_participation_report_parameter_sets::event_id.eq(events::id)),
        )
        .select((
            (
                events::all_columns,
                event_dates::all_columns.nullable(),
                event_recurrences::all_columns.nullable(),
            ),
            event_invites::all_columns.nullable(),
            rooms::all_columns,
            sip_configs::all_columns.nullable(),
            event_favorites::user_id.nullable().is_not_null(),
            event_shared_folders::all_columns.nullable(),
            tariffs::all_columns,
            event_training_participation_report_parameter_sets::all_columns.nullable(),
        ))
        .filter(users::disabled_since.is_null())
        .filter(events::id.eq(event_id))
        .first(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_with_room(
    conn: &mut DbConnection,
    event_id: EventId,
) -> Result<(EventRecord, Room, Option<SipConfig>)> {
    events::table
        .left_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .left_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .inner_join(rooms::table.on(events::room.eq(rooms::id)))
        .left_join(sip_configs::table.on(rooms::id.eq(sip_configs::room)))
        .select((
            (
                events::all_columns,
                event_dates::all_columns.nullable(),
                event_recurrences::all_columns.nullable(),
            ),
            rooms::all_columns,
            sip_configs::all_columns.nullable(),
        ))
        .filter(events::id.eq(event_id))
        .filter(users::disabled_since.is_null())
        .first(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
#[expect(clippy::too_many_arguments)]
pub async fn get_all_events_for_user_paginated_as_stream(
    conn: &mut DbConnection,
    user: User,
    only_favorites: bool,
    invite_status_filter: Vec<EventInviteStatus>,
    time_min: Option<DateTime<Utc>>,
    time_max: Option<DateTime<Utc>>,
    created_before: Option<DateTime<Utc>>,
    created_after: Option<DateTime<Utc>>,
    adhoc: Option<bool>,
    time_independent: Option<bool>,
    cursor: Option<GetEventsCursor>,
) -> Result<
    impl Stream<
        Item = Result<(
            EventRecord,
            Option<EventInvite>,
            Room,
            Option<SipConfig>,
            bool,
            Option<EventSharedFolder>,
            Tariff,
        )>,
    >,
> {
    // Validate that the event is either created by the given user or an invite to the event
    // exists for the user
    let event_related_to_user_id = events::created_by
        .eq(user.id)
        .or(event_invites::invitee.eq(user.id));

    // Create query which select events and joins into the room of the event
    let mut query = events::table
        .left_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .left_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .left_join(
            event_invites::table.on(event_invites::event_id
                .eq(events::id)
                .and(event_invites::invitee.eq(user.id))),
        )
        .left_join(
            event_favorites::table.on(event_favorites::event_id
                .eq(events::id)
                .and(event_favorites::user_id.eq(user.id))),
        )
        .left_join(event_shared_folders::table.on(event_shared_folders::event_id.eq(events::id)))
        .inner_join(rooms::table)
        .left_join(sip_configs::table.on(rooms::id.eq(sip_configs::room)))
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .inner_join(tariffs::table.on(tariffs::id.eq(users::tariff_id)))
        .select((
            (
                events::all_columns,
                event_dates::all_columns.nullable(),
                event_recurrences::all_columns.nullable(),
            ),
            event_invites::all_columns.nullable(),
            rooms::all_columns,
            sip_configs::all_columns.nullable(),
            event_favorites::user_id.nullable().is_not_null(),
            event_shared_folders::all_columns.nullable(),
            tariffs::all_columns,
        ))
        .filter(events::tenant_id.eq(user.tenant_id))
        .filter(event_related_to_user_id)
        .filter(users::disabled_since.is_null())
        .order_by(event_dates::starts_at.nullable().asc().nulls_first())
        .then_order_by(events::created_at.asc())
        .then_order_by(events::id.asc())
        .into_boxed::<Pg>();

    // Consider the start position as specified by the cursor
    if let Some(cursor) = cursor {
        if let Some(from_starts_at) = cursor.from_starts_at {
            let expr = AsExpression::<Record<(Timestamptz, Timestamptz, Uuid)>>::as_expression((
                event_dates::starts_at,
                events::created_at,
                events::id,
            ));

            // Get all records that are behind the cursor position.
            // Records with no start date are considered to be less than the cursor specifies because
            // they don't pass the '>' comparison below (a comparison with NULL is always NULL).
            query = query.filter(expr.gt((from_starts_at, cursor.from_created_at, cursor.from_id)));
        } else {
            let expr = AsExpression::<Record<(Timestamptz, Uuid)>>::as_expression((
                events::created_at,
                events::id,
            ));

            // Get all records that are behind the cursor position.
            // Records with a start date are considered to be greater than the cursor
            // specifies (which has no start date here).
            // For records without a start date the decision is based on the remaining values.
            query = query.filter(
                event_dates::starts_at
                    .is_not_null()
                    .or(event_dates::starts_at
                        .is_null()
                        .and(expr.gt((cursor.from_created_at, cursor.from_id)))),
            );
        }
    }

    // Add filters to query depending on the time_(min/max) parameters
    match (time_min, time_max) {
        (Some(time_min), Some(time_max)) => {
            // we have an overlap if any of these conditions matches:
            // - starts_at is between time_min and time_max
            // - ends_at is between time_min and time_max
            // - time_min is between starts_at and ends_at
            // - time_max is between starts_at and ends_at
            query = query.filter(
                event_dates::starts_at
                    .between(time_min, time_max)
                    .or(event_dates::ends_at.between(time_min, time_max))
                    .or(time_min
                        .into_sql::<Timestamptz>()
                        .between(event_dates::starts_at, event_dates::ends_at))
                    .or(time_max
                        .into_sql::<Timestamptz>()
                        .between(event_dates::starts_at, event_dates::ends_at)),
            );
        }
        (Some(time_min), None) => {
            query = query.filter(event_dates::ends_at.ge(time_min));
        }
        (None, Some(time_max)) => {
            query = query.filter(event_dates::starts_at.le(time_max));
        }
        (None, None) => {
            // no filters to apply
        }
    }

    if let Some(created_before) = created_before {
        query = query.filter(events::created_at.le(created_before));
    }

    if let Some(created_after) = created_after {
        query = query.filter(events::created_at.ge(created_after));
    }

    if only_favorites {
        query = query.filter(event_favorites::user_id.is_not_null());
    }

    if let Some(is_adhoc) = adhoc {
        query = query.filter(events::is_adhoc.eq(is_adhoc));
    }

    if let Some(is_time_independent) = time_independent {
        if is_time_independent {
            query = query.filter(event_dates::starts_at.is_null());
        } else {
            query = query.filter(event_dates::starts_at.is_not_null());
        }
    }

    if !invite_status_filter.is_empty() {
        if invite_status_filter.contains(&EventInviteStatus::Accepted) {
            // edge case to allow event creators to filter created events by 'accepted'
            query = query.filter(
                event_invites::status
                    .eq_any(invite_status_filter)
                    .or(event_invites::status.is_null()),
            );
        } else {
            query = query.filter(event_invites::status.eq_any(invite_status_filter));
        }
    }

    let stream = query
        .load_stream::<(
            EventRecord,
            Option<EventInvite>,
            Room,
            Option<SipConfig>,
            bool,
            Option<EventSharedFolder>,
            Tariff,
        )>(conn)
        .await?;

    Ok(convert_diesel_query_results(stream))
}

#[tracing::instrument(err(level = "debug"), skip_all)]
#[expect(clippy::too_many_arguments)]
pub async fn get_all_events_exceptions_for_user_paginated_as_stream(
    conn: &mut DbConnection,
    user: User,
    only_favorites: bool,
    invite_status_filter: Vec<EventInviteStatus>,
    time_min: Option<DateTime<Utc>>,
    time_max: Option<DateTime<Utc>>,
    created_before: Option<DateTime<Utc>>,
    created_after: Option<DateTime<Utc>>,
    adhoc: Option<bool>,
    time_independent: Option<bool>,
    cursor: Option<GetEventExceptionsCursor>,
) -> Result<impl Stream<Item = Result<(EventException, EventRecord)>>> {
    // Validate that the event is either created by the given user or an invite to the event
    // exists for the user
    let event_related_to_user_id = events::created_by
        .eq(user.id)
        .or(event_invites::invitee.eq(user.id));

    let mut query = event_exceptions::table
        .inner_join(events::table.on(event_exceptions::event_id.eq(events::id)))
        .inner_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .inner_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .left_join(
            event_invites::table.on(event_invites::event_id
                .eq(events::id)
                .and(event_invites::invitee.eq(user.id))),
        )
        .left_join(
            event_favorites::table.on(event_favorites::event_id
                .eq(events::id)
                .and(event_favorites::user_id.eq(user.id))),
        )
        .left_join(event_shared_folders::table.on(event_shared_folders::event_id.eq(events::id)))
        .inner_join(rooms::table.on(events::room.eq(rooms::id)))
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((
            event_exceptions::all_columns,
            (
                events::all_columns,
                event_dates::all_columns.nullable(),
                event_recurrences::all_columns.nullable(),
            ),
        ))
        .filter(events::tenant_id.eq(user.tenant_id))
        .filter(event_related_to_user_id)
        .filter(users::disabled_since.is_null())
        .order_by(event_exceptions::starts_at.nullable().asc().nulls_first())
        .then_order_by(event_exceptions::created_at.asc())
        .then_order_by(event_exceptions::event_id.asc())
        .then_order_by(event_exceptions::exception_date.asc())
        .into_boxed::<Pg>();

    // Consider the start position as specified by the cursor
    if let Some(cursor) = cursor {
        if let Some(from_starts_at) = cursor.from_starts_at {
            let expr = AsExpression::<
                Record<(Nullable<Timestamptz>, Timestamptz, Uuid, Timestamptz)>,
            >::as_expression((
                event_exceptions::starts_at,
                event_exceptions::created_at,
                event_exceptions::event_id,
                event_exceptions::exception_date,
            ));

            // Get all records that are behind the cursor position.
            // Records with no start date are considered to be less than the cursor specifies because
            // they don't pass the '>' comparison below (a comparison with NULL is always NULL).
            query = query.filter(expr.gt((
                from_starts_at,
                cursor.from_created_at,
                cursor.from_id,
                cursor.from_exception_date,
            )));
        } else {
            let expr = AsExpression::<Record<(Timestamptz, Uuid, Timestamptz)>>::as_expression((
                event_exceptions::created_at,
                event_exceptions::event_id,
                event_exceptions::exception_date,
            ));

            // Get all records that are behind the cursor position.
            // Records with a start date are considered to be greater than the cursor
            // specifies (which has no start date here).
            // For records without a start date the decision is based on the remaining values.
            query = query.filter(event_exceptions::starts_at.is_not_null().or(
                event_exceptions::starts_at.is_null().and(expr.gt((
                    cursor.from_created_at,
                    cursor.from_id,
                    cursor.from_exception_date,
                ))),
            ));
        }
    }

    // Add filters to query depending on the time_(min/max) parameters
    match (time_min, time_max) {
        (Some(time_min), Some(time_max)) => {
            // we have an overlap if any of these conditions matches:
            // - starts_at is between time_min and time_max
            // - ends_at is between time_min and time_max
            // - time_min is between starts_at and ends_at
            // - time_max is between starts_at and ends_at
            query = query.filter(
                event_dates::starts_at
                    .between(time_min, time_max)
                    .or(event_dates::ends_at.between(time_min, time_max))
                    .or(time_min
                        .into_sql::<Timestamptz>()
                        .between(event_dates::starts_at, event_dates::ends_at))
                    .or(time_max
                        .into_sql::<Timestamptz>()
                        .between(event_dates::starts_at, event_dates::ends_at)),
            );
        }
        (Some(time_min), None) => {
            query = query.filter(event_dates::ends_at.ge(time_min));
        }
        (None, Some(time_max)) => {
            query = query.filter(event_dates::starts_at.le(time_max));
        }
        (None, None) => {
            // no filters to apply
        }
    }

    if let Some(created_before) = created_before {
        query = query.filter(events::created_at.le(created_before));
    }

    if let Some(created_after) = created_after {
        query = query.filter(events::created_at.ge(created_after));
    }

    if only_favorites {
        query = query.filter(event_favorites::user_id.is_not_null());
    }

    if let Some(is_adhoc) = adhoc {
        query = query.filter(events::is_adhoc.eq(is_adhoc));
    }

    if let Some(is_time_independent) = time_independent {
        if is_time_independent {
            query = query.filter(event_dates::starts_at.is_null());
        } else {
            query = query.filter(event_dates::starts_at.is_not_null());
        }
    }

    if !invite_status_filter.is_empty() {
        if invite_status_filter.contains(&EventInviteStatus::Accepted) {
            // edge case to allow event creators to filter created events by 'accepted'
            query = query.filter(
                event_invites::status
                    .eq_any(invite_status_filter)
                    .or(event_invites::status.is_null()),
            );
        } else {
            query = query.filter(event_invites::status.eq_any(invite_status_filter));
        }
    }

    let stream = query
        .load_stream::<(EventException, EventRecord)>(conn)
        .await?;

    Ok(convert_diesel_query_results(stream))
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_events_for_user(
    conn: &mut DbConnection,
    user: User,
    only_recurring: bool,
) -> Result<Vec<EventRecord>> {
    // Filter applied to all events which validates that the event is either created by
    // the given user or an invite to the event exists for the user
    let event_related_to_user_id = events::created_by
        .eq(user.id)
        .or(event_invites::invitee.eq(user.id));

    // Create query which select events and joins into the room of the event
    let mut query = events::table
        .left_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .left_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .left_join(
            event_invites::table.on(event_invites::event_id
                .eq(events::id)
                .and(event_invites::invitee.eq(user.id))),
        )
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((
            events::all_columns,
            event_dates::all_columns.nullable(),
            event_recurrences::all_columns.nullable(),
        ))
        .filter(events::tenant_id.eq(user.tenant_id))
        .filter(event_related_to_user_id)
        .filter(users::disabled_since.is_null())
        .order_by(event_dates::starts_at.nullable().asc().nulls_first())
        .then_order_by(events::created_at.asc())
        .then_order_by(events::id.asc())
        .into_boxed::<Pg>();

    if only_recurring {
        query = query.filter(event_recurrences::recurrence_pattern.is_not_null());
    }

    query.load(conn).await.map_err(DatabaseError::from)
}

/// Select all room ids and their creator
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_all_event_and_creator_ids(
    conn: &mut DbConnection,
) -> Result<Vec<(EventId, UserId)>> {
    events::table
        .select((events::id, events::created_by))
        .load::<(EventId, UserId)>(conn)
        .await
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_by_id(conn: &mut DbConnection, event_id: EventId) -> Result<()> {
    let _ = diesel::delete(events::table)
        .filter(events::id.eq(event_id))
        .execute(conn)
        .await?;

    Ok(())
}

/// Returns the [`Event`] in the given [`RoomId`].
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_for_room(
    conn: &mut DbConnection,
    room: &RoomIdOrAlias,
) -> Result<Option<EventRecord>> {
    events::table
        .left_join(event_dates::table.on(event_dates::event_id.eq(events::id)))
        .left_join(
            event_recurrences::table.on(event_recurrences::event_id.eq(event_dates::event_id)),
        )
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select((
            events::all_columns,
            event_dates::all_columns.nullable(),
            event_recurrences::all_columns.nullable(),
        ))
        .inner_join(rooms::table.on(rooms::id.eq(events::room)))
        .filter_by_room(room)
        .filter(users::disabled_since.is_null())
        .first(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

/// Returns a [`EventId`] for the given [`RoomId`].
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_event_id_for_room(
    conn: &mut DbConnection,
    room_id: RoomId,
) -> Result<Option<EventId>> {
    events::table
        .inner_join(users::table.on(users::id.eq(events::created_by)))
        .select(events::id)
        .filter(events::room.eq(room_id))
        .filter(users::disabled_since.is_null())
        .first(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

/// Deletes all [`Event`]s in a given [`RoomId`]
///
/// Fastpath for deleting multiple events in room
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_event_for_room(conn: &mut DbConnection, room_id: RoomId) -> Result<()> {
    diesel::delete(events::table)
        .filter(events::room.eq(room_id))
        .execute(conn)
        .await?;

    Ok(())
}

/// Creates a new event.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn create_event(
    conn: &mut DbConnection,
    new_event_record: NewEventRecord,
) -> Result<EventRecord> {
    conn.transaction(async |conn| {
        let event: Event = diesel::insert_into(events::table)
            .values(new_event_record.event())
            .get_result(conn)
            .await?;

        let Some(new_event_date_record) = new_event_record.build_date(event.id) else {
            return Ok(EventRecord::new(event, None, None));
        };

        let date = Some(
            diesel::insert_into(event_dates::table)
                .values(new_event_date_record.date())
                .get_result(conn)
                .await?,
        );

        let Some(new_recurrence) = new_event_date_record.build_recurrence() else {
            return Ok(EventRecord::new(event, date, None));
        };

        let recurrence = Some(
            diesel::insert_into(event_recurrences::table)
                .values(new_recurrence)
                .get_result(conn)
                .await?,
        );

        Ok(EventRecord::new(event, date, recurrence))
    })
    .await
}

/// Update an event.
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn update_event(
    conn: &mut DbConnection,
    event_id: EventId,
    update_event_record: UpdateEventRecord,
) -> Result<EventRecord> {
    conn.transaction(async |conn| {
        let event = diesel::update(events::table)
            .filter(events::id.eq(event_id))
            .set((
                update_event_record.event(),
                events::revision.eq(events::revision + 1),
            ))
            .get_result(conn)
            .await?;

        let date = match update_event_record.date() {
            Some(update_date) => Some(
                diesel::insert_into(event_dates::table)
                    .values((event_dates::event_id.eq(event_id), update_date))
                    .on_conflict(event_dates::event_id)
                    .do_update()
                    .set(update_date)
                    .get_result(conn)
                    .await?,
            ),
            None => {
                let _ = diesel::delete(event_dates::table)
                    .filter(event_dates::event_id.eq(event_id))
                    .execute(conn)
                    .await?;

                None
            }
        };

        let recurrence = match update_event_record.recurrence() {
            Some(update_recurrence) => Some(
                diesel::insert_into(event_recurrences::table)
                    .values((event_recurrences::event_id.eq(event_id), update_recurrence))
                    .on_conflict(event_recurrences::event_id)
                    .do_update()
                    .set(update_recurrence)
                    .get_result(conn)
                    .await?,
            ),
            None => {
                let _ = diesel::delete(event_recurrences::table)
                    .filter(event_recurrences::event_id.eq(event_id))
                    .execute(conn)
                    .await?;

                None
            }
        };

        Ok(EventRecord::new(event, date, recurrence))
    })
    .await
}
