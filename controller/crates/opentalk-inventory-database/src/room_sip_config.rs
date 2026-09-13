// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_database::DatabaseError;
use opentalk_db_storage as db;
use opentalk_inventory::{
    NewRoomSipConfig, Room, RoomSipConfig, RoomSipConfigInventory, UpdateRoomSipConfig, User,
};
use opentalk_types_common::{
    call_in::CallInId,
    rooms::{RoomId, RoomIdOrAlias},
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl RoomSipConfigInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_room_sip_config(&mut self, room_id: RoomId) -> Result<Option<RoomSipConfig>> {
        match db::queries::sip_configs::get_room_sip_config(&mut self.inner, room_id).await {
            Ok(sip_config) => Ok(Some(sip_config.into())),
            Err(DatabaseError::NotFound) => Ok(None),
            Err(e) => Err(e).context(DatabaseSnafu)?,
        }
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_room_sip_config_with_room_and_creator(
        &mut self,
        call_in_id: CallInId,
    ) -> Result<Option<(RoomSipConfig, Room, User)>> {
        Ok(
            db::queries::sip_configs::get_room_sip_config_with_room_and_creator(
                &mut self.inner,
                &call_in_id,
            )
            .await
            .context(DatabaseSnafu)?
            .map(|(sip_config, room, user)| (sip_config.into(), room.into(), user.into())),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_room_sip_config(
        &mut self,
        sip_config: NewRoomSipConfig,
    ) -> Result<RoomSipConfig> {
        Ok(
            db::queries::sip_configs::create_room_sip_config(&mut self.inner, sip_config.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_room_sip_config(
        &mut self,
        room_id: RoomId,
        sip_config: UpdateRoomSipConfig,
    ) -> Result<Option<RoomSipConfig>> {
        Ok(db::queries::sip_configs::update_room_sip_config(
            &mut self.inner,
            sip_config.into(),
            room_id,
        )
        .await
        .context(DatabaseSnafu)?
        .map(Into::into))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_room_sip_config(&mut self, room: &RoomIdOrAlias) -> Result<()> {
        Ok(
            db::queries::sip_configs::delete_room_sip_config(&mut self.inner, room)
                .await
                .context(DatabaseSnafu)?,
        )
    }
}
