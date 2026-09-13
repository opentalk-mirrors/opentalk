// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{BTreeMap, BTreeSet};

use opentalk_types_common::{
    events::EventId,
    features::{FeatureId, ModuleFeatureId},
    modules::ModuleId,
    rooms::RoomIdOrAlias,
    users::UserId,
};

use crate::{Result, authorization::types::AuthorizationUserRole};

/// A trait for retrieving and storing event entities.
#[cfg_attr(feature = "mockall", mockall::automock)]
#[async_trait::async_trait]
pub trait AuthorizationInventory: Send {
    /// Returns the [`AuthorizationUserRole`] associated with the `user_id`.
    async fn get_event_user_role(
        &mut self,
        event_id: EventId,
        user_id: UserId,
        disabled_features: BTreeSet<ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Result<AuthorizationUserRole>;

    /// Return if guest access is allowed for the event (`event_id`).
    async fn get_event_guest_allowed(
        &mut self,
        event_id: EventId,
        disabled_features: BTreeSet<ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Result<bool>;

    /// Returns the [`AuthorizationUserRole`] associated with the `user_id` on the room.
    async fn get_room_user_role(
        &mut self,
        room_id_or_alias: &RoomIdOrAlias,
        user_id: UserId,
        disabled_features: BTreeSet<ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Result<AuthorizationUserRole>;

    /// Return if guest access is allowed for the room (`room_id`).
    async fn get_room_guest_allowed(
        &mut self,
        room_id_or_alias: &RoomIdOrAlias,
        disabled_features: BTreeSet<ModuleFeatureId>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Result<bool>;
}
