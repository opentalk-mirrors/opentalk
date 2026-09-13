// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use diesel::{
    expression::AppearsOnTable,
    pg::Pg,
    prelude::*,
    query_builder::{BoxedSelectStatement, FromClause},
    query_dsl::methods::BoxedDsl,
    query_source::QuerySource,
};
use opentalk_types_common::rooms::{RoomAlias, RoomIdOrAlias};

use crate::schema::rooms;

/// Restrict a query to a single room, identified either by its id or by its alias (name and optional suffix).
///
/// The query is boxed internally, so any clause that shapes the `SELECT` or `FROM` (e.g. `select`, `inner_join`) must
/// be applied before calling `filter_by_room`.
pub trait FilterByRoom<'a>: Sized {
    /// The boxed query type returned after applying the room filter.
    type Output;

    /// Box this query and restrict it to the room described by `room`.
    fn filter_by_room(self, room: &'a RoomIdOrAlias) -> Self::Output;
}

impl<'a, Q, ST, QS> FilterByRoom<'a> for Q
where
    Q: QueryDsl + BoxedDsl<'a, Pg, Output = BoxedSelectStatement<'a, ST, FromClause<QS>, Pg>>,
    QS: QuerySource,
    rooms::id: AppearsOnTable<QS>,
    rooms::name: AppearsOnTable<QS>,
    rooms::suffix: AppearsOnTable<QS>,
{
    type Output = BoxedSelectStatement<'a, ST, FromClause<QS>, Pg>;

    fn filter_by_room(self, room: &'a RoomIdOrAlias) -> Self::Output {
        let query = self.into_boxed();
        match room {
            RoomIdOrAlias::Id(id) => query.filter(rooms::id.eq(id)),
            // A room with a suffix: match name and suffix by value.
            RoomIdOrAlias::Alias(RoomAlias {
                name,
                suffix: Some(suffix),
            }) => query
                .filter(rooms::name.eq(name))
                .filter(rooms::suffix.eq(suffix)),
            // A room without a suffix (suffix disabled): the column is `NULL`, which must be matched with `IS NULL`
            // rather than `= NULL` (the latter never matches in SQL).
            RoomIdOrAlias::Alias(RoomAlias { name, suffix: None }) => query
                .filter(rooms::name.eq(name))
                .filter(rooms::suffix.is_null()),
        }
    }
}
