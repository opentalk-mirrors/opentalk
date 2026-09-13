// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides room-related implementation

use opentalk_controller_api_authorization::authorization::AuthorizationChange;
use opentalk_controller_service_facade::{RequestUser, StartRoomError};
use opentalk_controller_utils::{
    CaptureApiError,
    deletion::{Deleter, RoomDeleter},
};
use opentalk_inventory::{
    NewRoom, NewRoomSipConfig, Room, UpdateRoom, utils::is_room_guest_access_allowed,
};
use opentalk_types_api_v1::{
    error::ApiError,
    pagination::PagePaginationQuery,
    rooms::{
        GetRoomsResponseBody, RoomResource, by_room_id::GetRoomEventResponseBody,
        name::PostRoomNameVerifyResponseBody,
    },
};
use opentalk_types_common::{
    events::EventInfo,
    features::GUESTS_ALLOWED_MODULE_FEATURE_ID,
    pagination::ItemCount,
    rooms::{GuestAccess, RoomAlias, RoomId, RoomIdOrAlias, RoomName, RoomPassword},
    users::UserId,
};

use crate::{
    ControllerBackend, ToUserProfile,
    controller_backend::utils::{build_room_alias, ensure_guest_access_valid, resolve_room_id},
};

impl ControllerBackend {
    pub(crate) async fn get_rooms(
        &self,
        current_user_id: UserId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsResponseBody, ItemCount), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let (rooms, room_count) = inventory
            .get_rooms_accessible_to_user_with_creator_paginated(
                current_user_id,
                pagination.per_page,
                pagination.page,
            )
            .await?;

        let rooms = rooms
            .into_iter()
            .map(|(room, user)| RoomResource {
                id: room.id,
                alias: room.alias,
                created_by: user.to_public_user_profile(&settings),
                created_at: room.created_at,
                password: room.password,
                waiting_room: room.waiting_room,
                guest_access: room.guest_access,
            })
            .collect::<Vec<RoomResource>>();

        Ok((GetRoomsResponseBody(rooms), room_count))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn create_room(
        &self,
        current_user: RequestUser,
        name: Option<RoomName>,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        guest_access: Option<GuestAccess>,
        e2e_encryption: bool,
    ) -> Result<RoomResource, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let tariff = self.get_tariff_for_user(current_user.id).await?;
        let guest_access = guest_access.unwrap_or(GuestAccess::WaitingRoom);

        if enable_sip {
            Self::ensure_call_in_permission(e2e_encryption, guest_access, &tariff)?
        }
        ensure_guest_access_valid(guest_access, e2e_encryption, &tariff)?;

        let alias = build_room_alias(name, &settings);

        let room = inventory
            .create_room(NewRoom {
                created_by: current_user.id,
                alias,
                password,
                waiting_room,
                guest_access,
                e2e_encryption,
                tenant_id: current_user.tenant_id,
            })
            .await?;

        if enable_sip {
            _ = inventory
                .create_room_sip_config(NewRoomSipConfig::new(room.id, false))
                .await?;
        }

        drop(inventory);

        let room_resource = RoomResource {
            id: room.id,
            alias: room.alias,
            created_by: current_user.to_public_user_profile(&settings),
            created_at: room.created_at,
            password: room.password,
            waiting_room: room.waiting_room,
            guest_access: room.guest_access,
        };

        let is_guest_feature_enabled = tariff.has_feature_enabled(
            &GUESTS_ALLOWED_MODULE_FEATURE_ID.module,
            &GUESTS_ALLOWED_MODULE_FEATURE_ID.feature,
        );
        self.authorizer
            .apply_change(&AuthorizationChange::CreateRoom {
                room: room_resource.id,
                creator: current_user.id,
                is_guest_feature_enabled,
                guest_access: room_resource.guest_access,
                e2e_encryption,
            })
            .await
            .map_err(|e| {
                log::error!("Could not apply changes in the authorization database: {e:?}");
                ApiError::internal()
            })?;

        Ok(room_resource)
    }

    pub(crate) async fn patch_room(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        name: Option<Option<RoomName>>,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        guest_access: Option<GuestAccess>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, CaptureApiError> {
        let room = self
            .update_room(
                room_id_or_alias,
                name,
                password,
                waiting_room,
                guest_access,
                e2e_encryption,
            )
            .await?;

        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;
        let created_by = inventory
            .get_user(room.created_by)
            .await?
            .to_public_user_profile(&settings);

        let room_resource = RoomResource {
            id: room.id,
            alias: room.alias,
            created_by,
            created_at: room.created_at,
            password: room.password,
            waiting_room: room.waiting_room,
            guest_access: room.guest_access,
        };

        Ok(room_resource)
    }

    /// Updates a room in the database and applies the necessary changes in the authorization middleware.
    pub(crate) async fn update_room(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        name: Option<Option<RoomName>>,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        guest_access: Option<GuestAccess>,
        e2e_encryption: Option<bool>,
    ) -> Result<Room, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;
        let settings = self.settings_provider.get();

        let alias = name.map(|name| build_room_alias(name, &settings));

        if guest_access.is_some() || e2e_encryption.is_some() {
            let room = inventory.get_room(&room_id_or_alias).await?;
            let tariff = self.get_tariff_for_user(room.created_by).await?;

            let guest_access = guest_access.unwrap_or(room.guest_access);
            let e2e_encryption = e2e_encryption.unwrap_or(room.e2e_encryption);

            ensure_guest_access_valid(guest_access, e2e_encryption, &tariff)?;
        }

        let room = inventory
            .update_room(
                &room_id_or_alias,
                UpdateRoom {
                    alias,
                    password,
                    waiting_room,
                    guest_access,
                    e2e_encryption,
                },
            )
            .await?;

        self.authorizer
            .apply_change(&AuthorizationChange::UpdateRoomConfiguration {
                room: room.id,
                guest_access,
                e2e_encryption,
            })
            .await?;

        Ok(room)
    }

    pub(crate) async fn delete_room(
        &self,
        current_user: RequestUser,
        room_id_or_alias: RoomIdOrAlias,
        force_delete_reference_if_external_services_fail: bool,
    ) -> Result<(), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        // Resolve the room alias to a room id here instead of piping the `RoomIdOrAlias` through the `RoomDeleter`
        // because the `RoomDeleter` interacts with the auth API, which only accepts room ids, not aliases.
        let room_id = resolve_room_id(inventory.as_mut(), &room_id_or_alias).await?;
        let deleter = RoomDeleter::new(room_id, force_delete_reference_if_external_services_fail);

        deleter
            .perform(
                log::logger(),
                inventory.as_mut(),
                self.authorizer.clone(),
                self.roomserver.as_ref(),
                Some(current_user.id),
                &settings,
                &self.storage,
            )
            .await?;

        Ok(())
    }

    pub(crate) async fn verify_room_name(
        &self,
        name: RoomName,
    ) -> Result<PostRoomNameVerifyResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();

        // When the suffix is enabled, we can assume that the room name is available because the alias will be unique.
        if !settings.defaults.room_alias.disable_suffix {
            return Ok(PostRoomNameVerifyResponseBody { available: true });
        }

        let mut inventory = self.inventory_provider.get_inventory().await?;
        let exists = inventory
            .exists_room(&RoomAlias { name, suffix: None }.into())
            .await?;

        Ok(PostRoomNameVerifyResponseBody { available: !exists })
    }

    pub(crate) async fn get_room(
        &self,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<RoomResource, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let (room, created_by) = inventory.get_room_with_creator(&room_id_or_alias).await?;

        let room_resource = RoomResource {
            id: room.id,
            alias: room.alias,
            created_by: created_by.to_public_user_profile(&settings),
            created_at: room.created_at,
            password: room.password,
            waiting_room: room.waiting_room,
            guest_access: room.guest_access,
        };

        Ok(room_resource)
    }

    pub(crate) async fn get_room_event(
        &self,
        current_user: Option<RequestUser>,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<GetRoomEventResponseBody, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let room = inventory.get_room(&room_id_or_alias).await?;
        let event = inventory.get_event_for_room(&room.id.into()).await?;
        let Some(event) = event.as_ref() else {
            return Err(ApiError::not_found().into());
        };

        // Naive check that prevents joining events that were created by a user which is since been
        // disabled. The `get_user` method returns 404 not found when the user is disabled.
        if inventory.get_user(room.created_by).await.is_err() {
            return Err(ApiError::forbidden().into());
        }

        let is_privileged_user = match &current_user {
            Some(user) => {
                user.id == room.created_by
                    || inventory
                        .get_event_invite_for_user_and_room(user.id, room.id)
                        .await?
                        .is_some()
            }
            None => false,
        };

        let event_info = EventInfo {
            id: event.id,
            room_id: event.room,
            title: event.title.clone(),
            is_adhoc: event.is_adhoc,
            e2e_encryption: room.e2e_encryption,
            password_required: room.password.is_some() && !is_privileged_user,
        };

        Ok(GetRoomEventResponseBody(event_info))
    }

    /// Check the room password and guest access
    ///
    /// Returns the associated room
    pub(crate) async fn authenticate_guest(
        &self,
        room_id: RoomId,
        password: Option<RoomPassword>,
    ) -> Result<Room, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;
        let (room, created_by) = inventory.get_room_with_creator(&room_id.into()).await?;
        let tariff = self.get_tariff_for_user(created_by.id).await?;

        if !is_room_guest_access_allowed(&room, &tariff) {
            // Don't leak the existence of the room
            return Err(ApiError::not_found().into());
        }

        drop(inventory);

        match (&room.password, &password) {
            (Some(_), None) => Err(StartRoomError::WrongRoomPassword.into()),
            (Some(room_password), Some(password)) if password != room_password => {
                Err(StartRoomError::WrongRoomPassword.into())
            }
            _ => Ok(room),
        }
    }
}
