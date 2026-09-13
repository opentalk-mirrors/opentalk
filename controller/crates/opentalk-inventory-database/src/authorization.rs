// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use opentalk_db_storage as db;
use opentalk_inventory::{
    AuthorizationInventory, AuthorizationUserRole as Role, Room, Tariff, utils,
};
use opentalk_types_common::{
    events::EventId,
    features::{FeatureId, ModuleFeatureId},
    modules::ModuleId,
    rooms::RoomIdOrAlias,
    users::UserId,
};
use snafu::ResultExt as _;

use crate::{DatabaseConnection, Result, error::DatabaseSnafu};

#[async_trait::async_trait]
impl AuthorizationInventory for DatabaseConnection {
    async fn get_event_user_role(
        &mut self,
        event_id: EventId,
        user_id: UserId,
        disabled_features: BTreeSet<ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Result<Role> {
        let is_owner =
            db::queries::authorization::events::is_event_owner(&mut self.inner, event_id, user_id)
                .await
                .context(DatabaseSnafu)?;

        if is_owner {
            return Ok(Role::Owner);
        }

        let role = db::queries::authorization::events::get_event_user_role(
            &mut self.inner,
            event_id,
            user_id,
        )
        .await
        .context(DatabaseSnafu)?;

        if let Some(role) = role {
            return Ok(Role::Invited(role));
        }

        let guest_access = self
            .get_event_guest_allowed(event_id, disabled_features, module_features)
            .await?;

        Ok(Role::Unrelated { guest_access })
    }

    async fn get_event_guest_allowed(
        &mut self,
        event_id: EventId,
        disabled_features: BTreeSet<ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Result<bool> {
        let room_invite_and_tariff = db::queries::authorization::events::get_event_room_and_tariff(
            &mut self.inner,
            event_id,
        )
        .await
        .context(DatabaseSnafu)?;

        let Some((room, tariff)) = room_invite_and_tariff else {
            return Ok(false);
        };

        let room = Room::from(room);
        let tariff = Tariff::from(tariff).to_tariff_resource(disabled_features, module_features);

        if utils::is_room_guest_access_allowed(&room, &tariff) {
            return Ok(true);
        }

        Ok(false)
    }

    async fn get_room_user_role(
        &mut self,
        room_id_or_alias: &RoomIdOrAlias,
        user_id: UserId,
        disabled_features: BTreeSet<ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Result<Role> {
        let is_owner = db::queries::authorization::rooms::is_room_owner(
            &mut self.inner,
            room_id_or_alias,
            user_id,
        )
        .await
        .context(DatabaseSnafu)?;

        if is_owner {
            return Ok(Role::Owner);
        }

        let role = db::queries::authorization::rooms::get_room_user_role(
            &mut self.inner,
            room_id_or_alias,
            user_id,
        )
        .await
        .context(DatabaseSnafu)?;

        if let Some(role) = role {
            return Ok(Role::Invited(role));
        }

        let guest_access = self
            .get_room_guest_allowed(room_id_or_alias, disabled_features, module_features)
            .await?;

        Ok(Role::Unrelated { guest_access })
    }

    async fn get_room_guest_allowed(
        &mut self,
        room_id_or_alias: &RoomIdOrAlias,
        disabled_features: BTreeSet<ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Result<bool> {
        let room_invite_and_tariff = db::queries::authorization::rooms::get_room_and_tariff(
            &mut self.inner,
            room_id_or_alias,
        )
        .await
        .context(DatabaseSnafu)?;

        let Some((room, tariff)) = room_invite_and_tariff else {
            return Ok(false);
        };

        let room = Room::from(room);
        let tariff = Tariff::from(tariff).to_tariff_resource(disabled_features, module_features);

        if utils::is_room_guest_access_allowed(&room, &tariff) {
            return Ok(true);
        }

        Ok(false)
    }
}
