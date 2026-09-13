// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Insertable;
use opentalk_types_common::{events::EventId, users::UserId};

use crate::schema::event_favorites;

#[derive(Insertable)]
#[diesel(table_name = event_favorites)]
pub struct NewEventFavorite {
    pub user_id: UserId,
    pub event_id: EventId,
}
