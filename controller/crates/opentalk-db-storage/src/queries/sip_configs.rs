// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains sip configs database queries

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use opentalk_database::{DatabaseError, DbConnection, Result};
use opentalk_types_common::{
    call_in::CallInId,
    rooms::{RoomId, RoomIdOrAlias},
};

use crate::{
    queries::room_filter::FilterByRoom,
    schema::{rooms, sip_configs, users},
    tables::{
        rooms::Room,
        sip_configs::{NewSipConfig, SipConfig, UpdateSipConfig},
        users::User,
    },
};

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room_sip_config_with_room_and_creator(
    conn: &mut DbConnection,
    sip_id: &CallInId,
) -> Result<Option<(SipConfig, Room, User)>> {
    let query = sip_configs::table
        .filter(sip_configs::sip_id.eq(sip_id))
        .inner_join(rooms::table)
        .inner_join(users::table.on(users::id.eq(rooms::created_by)));

    Ok(query.get_result(conn).await.optional()?)
}

/// Get the sip config for the specified room
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn get_room_sip_config(conn: &mut DbConnection, room_id: RoomId) -> Result<SipConfig> {
    sip_configs::table
        .filter(sip_configs::room.eq(&room_id))
        .get_result(conn)
        .await
        .map_err(DatabaseError::from)
}

/// Delete the sip config for the specified room
#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn delete_room_sip_config(conn: &mut DbConnection, room: &RoomIdOrAlias) -> Result<()> {
    _ = diesel::delete(
        sip_configs::table
            .filter(sip_configs::room.eq_any(rooms::table.select(rooms::id).filter_by_room(room))),
    )
    .execute(conn)
    .await?;

    Ok(())
}

pub async fn update_room_sip_config(
    conn: &mut DbConnection,
    update_sip_config: UpdateSipConfig,
    room_id: RoomId,
) -> Result<Option<SipConfig>> {
    diesel::update(sip_configs::table.filter(sip_configs::room.eq(&room_id)))
        .set(update_sip_config)
        .get_result(conn)
        .await
        .optional()
        .map_err(DatabaseError::from)
}

#[tracing::instrument(err(level = "debug"), skip_all)]
pub async fn create_room_sip_config(
    conn: &mut DbConnection,
    mut new_sip_config: NewSipConfig,
) -> Result<SipConfig> {
    const ATTEMPTS: u8 = 3;

    for _ in 0..ATTEMPTS {
        let query = diesel::insert_into(sip_configs::table).values(&new_sip_config);

        let config = match query.get_result(conn).await {
            Ok(config) => config,
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => {
                new_sip_config.sip_id = CallInId::generate();
                continue;
            }
            Err(e) => return Err(e.into()),
        };

        return Ok(config);
    }

    Err(DatabaseError::Custom {
        message: format!(
            "Failed to insert new sip config for room {} {} times (collision)",
            new_sip_config.room, ATTEMPTS
        ),
    })
}
