// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Queryable;
use opentalk_types_common::{events::EventId, users::UserId};

use crate::{
    schema::event_favorites,
    tables::{events::Event, users::User},
};

#[derive(Associations, Identifiable, Queryable)]
#[diesel(table_name = event_favorites)]
#[diesel(primary_key(user_id, event_id))]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Event))]
pub struct EventFavorite {
    pub user_id: UserId,
    pub event_id: EventId,
}
