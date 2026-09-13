// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_db_storage as db;
use opentalk_inventory::{NewRoom, Room, RoomInventory, UpdateRoom, User};
use opentalk_types_common::{
    pagination::{ItemCount, Page, PageSize},
    rooms::{RoomId, RoomIdOrAlias},
    users::UserId,
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl RoomInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_room(&mut self, new_room: NewRoom) -> Result<Room> {
        Ok(
            db::queries::rooms::create_room(&mut self.inner, new_room.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_room(&mut self, room: &RoomIdOrAlias) -> Result<Room> {
        Ok(db::queries::rooms::get_room(&mut self.inner, room)
            .await
            .context(DatabaseSnafu)?
            .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn exists_room(&mut self, room: &RoomIdOrAlias) -> Result<bool> {
        Ok(db::queries::rooms::exists_room(&mut self.inner, room)
            .await
            .context(DatabaseSnafu)?)
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_room_with_creator(&mut self, room: &RoomIdOrAlias) -> Result<(Room, User)> {
        let (room, user) = db::queries::rooms::get_room_with_creator(&mut self.inner, room)
            .await
            .context(DatabaseSnafu)?;
        Ok((room.into(), user.into()))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_room(&mut self, room: &RoomIdOrAlias, update: UpdateRoom) -> Result<Room> {
        Ok(
            db::queries::rooms::update_room(&mut self.inner, update.into(), room)
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_room(&mut self, room_id: RoomId) -> Result<()> {
        Ok(db::queries::rooms::delete_room(&mut self.inner, room_id)
            .await
            .context(DatabaseSnafu)?)
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_orphaned_room_ids(&mut self) -> Result<Vec<RoomId>> {
        Ok(
            db::queries::rooms::get_all_orphaned_room_ids(&mut self.inner)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err, skip_all)]
    async fn get_rooms_accessible_to_user_with_creator_paginated(
        &mut self,
        user: UserId,
        limit: PageSize,
        page: Page,
    ) -> Result<(Vec<(Room, User)>, ItemCount)> {
        let (items, overall) = db::queries::rooms::get_accessible_to_user_with_creator_paginated(
            &mut self.inner,
            user,
            limit,
            page,
        )
        .await
        .context(DatabaseSnafu)?;
        Ok((
            items
                .into_iter()
                .map(|(room, user)| (room.into(), user.into()))
                .collect(),
            overall,
        ))
    }
}
