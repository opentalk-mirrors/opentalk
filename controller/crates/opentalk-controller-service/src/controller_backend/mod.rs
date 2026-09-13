// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides the default [`OpenTalkControllerService`] implementation.
mod assets;
mod auth;
mod events;
mod invites;
mod module_resources;
pub mod rooms;

pub mod roomserver;
mod services;
mod sip_configs;
mod streaming_targets;
mod tariff;
mod users;
mod utils;

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use openidconnect::AccessToken;
use opentalk_asset_storage::{
    AssetSaved, ByStreamExt, NewAssetFileName, ObjectStorage, ObjectStorageError, StorageNotifier,
    asset_key,
};
use opentalk_controller_api_authorization::authorization::Authorizer;
use opentalk_controller_service_facade::{
    AssetDownloadProxyStream, OpenTalkControllerService, RequestUser,
};
use opentalk_controller_settings::SettingsProvider;
use opentalk_inventory::InventoryProvider;
use opentalk_keycloak_admin::KeycloakAdminClient;
use opentalk_types_api_internal::{
    call_in::PostCallInStartRoomServerRequestBody,
    module_assets::AssetResource,
    module_resources::{
        ModuleResource, ModuleResourceFilter, ModuleResourceOperation, NewModuleResource,
    },
    recording::RecordingTarget,
};
use opentalk_types_api_v1::{
    assets::AssetSortingQuery,
    auth::{
        GetLoginResponseBody, LogoutToken, OidcProvider, PostLoginResponseBody,
        login::AuthLoginPostRequestBody,
    },
    error::ApiError,
    events::{
        DeleteEventInvitePath, DeleteEventsQuery, DeleteSharedFolderQuery, EventInstance,
        EventInstancePath, EventInstanceQuery, EventInvitee, EventOptionsQuery, EventOrException,
        EventOrInstance, EventResource, GetEventInstanceResponseBody, GetEventInstancesQuery,
        GetEventInstancesResponseBody, GetEventQuery, GetEventsAndInstancesQuery, GetEventsQuery,
        PatchEmailInviteBody, PatchEventBody, PatchEventInstanceBody, PatchEventQuery,
        PatchInviteBody, PostEventInviteBody, PostEventInviteQuery, PostEventsBody,
        PutSharedFolderQuery, StreamingTargetOptionsQuery,
        by_event_id::invites::GetEventsInvitesQuery,
    },
    pagination::PagePaginationQuery,
    rooms::{
        GetRoomsResponseBody, RoomResource,
        by_room_id::{
            GetRoomEventResponseBody, PostRoomsRoomserverStartRequestBody,
            RoomserverStartResponseBody,
            assets::RoomsByRoomIdAssetsGetResponseBody,
            invites::{PostInviteVerifyRequestBody, PostInviteVerifyResponseBody},
            sip::{PutSipConfigRequestBody, SipConfigResource},
            streaming_targets::{
                GetRoomStreamingTargetResponseBody, GetRoomStreamingTargetsResponseBody,
                PatchRoomStreamingTargetRequestBody, PatchRoomStreamingTargetResponseBody,
                PostRoomStreamingTargetResponseBody, RoomAndStreamingTargetId,
            },
        },
        name::PostRoomNameVerifyResponseBody,
    },
    users::{
        GetEventInvitesPendingResponseBody, GetFindQuery, GetFindResponseBody,
        GetUserAssetsResponseBody, PrivateUserProfile, PublicUserProfile, me::PatchMeRequestBody,
    },
};
use opentalk_types_common::{
    assets::AssetId,
    email::EmailAddress,
    events::EventId,
    features::FeatureId,
    modules::ModuleId,
    pagination::{ItemCount, Page, PageSize},
    rooms::{GuestAccess, RoomIdOrAlias, RoomName, RoomPassword},
    shared_folders::SharedFolder,
    streaming::StreamingTarget,
    tariffs::TariffResource,
    users::UserId,
};
use url::Url;

pub use crate::controller_backend::events::shared_folder::{
    delete_shared_folders, put_shared_folder,
};
use crate::{
    controller_backend::roomserver::RoomServerBackend,
    oidc::{Cache, OidcTokenHandler},
    services::MailService,
};

/// The default [`OpenTalkControllerService`] implementation.
pub struct ControllerBackend {
    settings_provider: SettingsProvider,
    authorizer: Authorizer,
    inventory_provider: Arc<dyn InventoryProvider>,
    oidc_cache: Arc<Cache>,
    oidc_token_handler: Arc<dyn OidcTokenHandler>,
    frontend_oidc_provider: OidcProvider,
    storage: Arc<ObjectStorage>,
    mail_service: Arc<Option<MailService>>,
    user_search_client: Arc<Option<KeycloakAdminClient>>,
    module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    roomserver: Arc<dyn RoomServerBackend>,
}

impl ControllerBackend {
    /// Create a new [`ControllerBackend`].
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        settings_provider: SettingsProvider,
        authorizer: Authorizer,
        inventory_provider: Arc<dyn InventoryProvider>,
        oidc_cache: Arc<Cache>,
        oidc_token_handler: Arc<dyn OidcTokenHandler>,
        frontend_oidc_provider: OidcProvider,
        storage: Arc<ObjectStorage>,
        mail_service: Arc<Option<MailService>>,
        user_search_client: Arc<Option<KeycloakAdminClient>>,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
        roomserver: Arc<dyn RoomServerBackend>,
    ) -> Self {
        Self {
            settings_provider,
            authorizer,
            inventory_provider,
            oidc_cache,
            oidc_token_handler,
            frontend_oidc_provider,
            storage,
            mail_service,
            user_search_client,
            module_features,
            roomserver,
        }
    }
}

impl std::fmt::Debug for ControllerBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ControllerBackend")
    }
}

#[async_trait(?Send)]
impl OpenTalkControllerService for ControllerBackend {
    async fn get_login(&self) -> GetLoginResponseBody {
        self.get_login().await
    }

    async fn post_login(
        &self,
        body: AuthLoginPostRequestBody,
    ) -> Result<PostLoginResponseBody, ApiError> {
        Ok(self.post_login(body).await?)
    }

    async fn post_logout(&self, logout_token: &LogoutToken) -> Result<(), ApiError> {
        Ok(self.post_logout(logout_token).await?)
    }

    async fn get_rooms(
        &self,
        current_user_id: UserId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsResponseBody, ItemCount), ApiError> {
        Ok(self.get_rooms(current_user_id, pagination).await?)
    }

    async fn create_room(
        &self,
        current_user: RequestUser,
        name: Option<RoomName>,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        guest_access: Option<GuestAccess>,
        e2e_encryption: bool,
    ) -> Result<RoomResource, ApiError> {
        Ok(self
            .create_room(
                current_user,
                name,
                password,
                enable_sip,
                waiting_room,
                guest_access,
                e2e_encryption,
            )
            .await?)
    }

    async fn patch_room(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        name: Option<Option<RoomName>>,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        guest_access: Option<GuestAccess>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, ApiError> {
        Ok(self
            .patch_room(
                room_id_or_alias,
                name,
                password,
                waiting_room,
                guest_access,
                e2e_encryption,
            )
            .await?)
    }

    async fn delete_room(
        &self,
        current_user: RequestUser,
        room_id_or_alias: RoomIdOrAlias,
        force_delete_reference_if_external_services_fail: bool,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_room(
                current_user,
                room_id_or_alias,
                force_delete_reference_if_external_services_fail,
            )
            .await?)
    }

    async fn verify_room_name(
        &self,
        name: RoomName,
    ) -> Result<PostRoomNameVerifyResponseBody, ApiError> {
        Ok(self.verify_room_name(name).await?)
    }

    async fn get_room(&self, room_id_or_alias: RoomIdOrAlias) -> Result<RoomResource, ApiError> {
        Ok(self.get_room(room_id_or_alias).await?)
    }

    async fn get_room_tariff(
        &self,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<TariffResource, ApiError> {
        Ok(self.get_room_tariff(room_id_or_alias).await?)
    }

    async fn get_room_event(
        &self,
        current_user: Option<RequestUser>,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<GetRoomEventResponseBody, ApiError> {
        Ok(self.get_room_event(current_user, room_id_or_alias).await?)
    }

    async fn start_room_session(
        &self,
        current_user: Option<RequestUser>,
        room_id_or_alias: RoomIdOrAlias,
        request: PostRoomsRoomserverStartRequestBody,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, ApiError> {
        Ok(self
            .start_room(current_user, room_id_or_alias, request, host)
            .await?)
    }

    async fn start_recording(
        &self,
        body: RecordingTarget,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, ApiError> {
        Ok(self.start_recording_impl(body, host).await?)
    }

    async fn start_transcription(
        &self,
        body: RecordingTarget,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, ApiError> {
        Ok(self.start_transcription_impl(body, host).await?)
    }

    async fn start_call_in(
        &self,
        request: PostCallInStartRoomServerRequestBody,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, ApiError> {
        Ok(self.start_call_in_roomserver_impl(request, host).await?)
    }

    async fn get_room_assets(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        pagination: &PagePaginationQuery,
    ) -> Result<(RoomsByRoomIdAssetsGetResponseBody, ItemCount), ApiError> {
        Ok(self.get_room_assets(room_id_or_alias, pagination).await?)
    }

    async fn get_room_asset(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<ByStreamExt, ApiError> {
        Ok(self.get_room_asset(room_id_or_alias, asset_id).await?)
    }

    async fn get_room_asset_proxy_download_token(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<String, ApiError> {
        Ok(self
            .get_room_asset_proxy_download_token(room_id_or_alias, asset_id)
            .await?)
    }

    async fn get_asset_proxy_download_stream(
        &self,
        asset_id: AssetId,
        token: String,
        range_header: Option<String>,
    ) -> Result<AssetDownloadProxyStream, ApiError> {
        let stream = self
            .storage
            .get_proxied(&asset_key(&asset_id), token, range_header)
            .await?;

        Ok(AssetDownloadProxyStream {
            status: stream.status,
            headers: stream.headers,
            stream: stream.stream,
        })
    }

    async fn create_room_asset(
        &self,
        storage_notifier: &dyn StorageNotifier,
        room_id_or_alias: RoomIdOrAlias,
        filename: NewAssetFileName,
        namespace: Option<ModuleId>,
        data: Box<dyn Stream<Item = Result<Bytes, ObjectStorageError>> + Unpin>,
    ) -> Result<(AssetResource, AssetSaved), ApiError> {
        Ok(self
            .create_room_asset(
                storage_notifier,
                room_id_or_alias,
                filename,
                namespace,
                data,
            )
            .await?)
    }

    async fn delete_room_asset(
        &self,
        storage_notifier: &dyn StorageNotifier,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_room_asset(storage_notifier, room_id_or_alias, asset_id)
            .await?)
    }

    async fn new_event(
        &self,
        current_user: RequestUser,
        event: PostEventsBody,
        query: EventOptionsQuery,
    ) -> Result<EventResource, ApiError> {
        Ok(self.new_event(current_user, event, query).await?)
    }

    async fn get_events_and_exceptions_interwoven(
        &self,
        current_user: RequestUser,
        query: GetEventsQuery,
    ) -> Result<(Vec<EventOrException>, Option<String>, Option<String>), ApiError> {
        Ok(self
            .get_events_and_exceptions_interwoven(current_user, query)
            .await?)
    }

    async fn get_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: GetEventQuery,
    ) -> Result<EventResource, ApiError> {
        Ok(self.get_event(current_user, event_id, query).await?)
    }

    async fn patch_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PatchEventQuery,
        patch: PatchEventBody,
    ) -> Result<Option<EventResource>, ApiError> {
        Ok(self
            .patch_event(current_user, event_id, query, patch)
            .await?)
    }

    async fn delete_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteEventsQuery,
    ) -> Result<(), ApiError> {
        Ok(self.delete_event(current_user, event_id, query).await?)
    }

    /// Get a list of events and instances
    async fn get_events_and_instances_interwoven(
        &self,
        current_user: RequestUser,
        query: GetEventsAndInstancesQuery,
    ) -> Result<(Vec<EventOrInstance>, Option<String>, Option<String>), ApiError> {
        Ok(self
            .get_events_and_instances_interwoven(current_user, query)
            .await?)
    }

    async fn get_event_instances(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        query: GetEventInstancesQuery,
    ) -> Result<
        (
            GetEventInstancesResponseBody,
            Option<String>,
            Option<String>,
        ),
        ApiError,
    > {
        Ok(self
            .get_event_instances(current_user, event_id, query)
            .await?)
    }

    async fn get_event_instance(
        &self,
        current_user: &RequestUser,
        path: EventInstancePath,
        query: EventInstanceQuery,
    ) -> Result<GetEventInstanceResponseBody, ApiError> {
        Ok(self.get_event_instance(current_user, path, query).await?)
    }

    async fn patch_event_instance(
        &self,
        current_user: RequestUser,
        path: EventInstancePath,
        query: EventInstanceQuery,
        patch: PatchEventInstanceBody,
    ) -> Result<Option<EventInstance>, ApiError> {
        Ok(self
            .patch_event_instance(current_user, path, query, patch)
            .await?)
    }

    async fn get_invites_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: GetEventsInvitesQuery,
    ) -> Result<(Vec<EventInvitee>, PageSize, Page, ItemCount), ApiError> {
        Ok(self
            .get_invites_for_event(current_user, event_id, query)
            .await?)
    }

    async fn create_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PostEventInviteQuery,
        create_invite: PostEventInviteBody,
    ) -> Result<bool, ApiError> {
        Ok(self
            .create_invite_to_event(current_user, event_id, query, create_invite)
            .await?)
    }

    async fn update_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        user_id: UserId,
        update_invite: &PatchInviteBody,
    ) -> Result<(), ApiError> {
        Ok(self
            .update_invite_to_event(current_user, event_id, user_id, update_invite)
            .await?)
    }

    async fn update_email_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        update_invite: &PatchEmailInviteBody,
    ) -> Result<(), ApiError> {
        Ok(self
            .update_email_invite_to_event(current_user, event_id, update_invite)
            .await?)
    }

    async fn delete_invite_to_event(
        &self,
        current_user: RequestUser,
        path: DeleteEventInvitePath,
        query: EventOptionsQuery,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_invite_to_event(current_user, path, query)
            .await?)
    }

    async fn delete_email_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        email: EmailAddress,
        query: EventOptionsQuery,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_email_invite_to_event(current_user, event_id, email, query)
            .await?)
    }

    async fn get_event_invites_pending(
        &self,
        user_id: UserId,
    ) -> Result<GetEventInvitesPendingResponseBody, ApiError> {
        Ok(self.get_event_invites_pending(user_id).await?)
    }

    async fn accept_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), ApiError> {
        Ok(self.accept_event_invite(user_id, event_id).await?)
    }

    async fn decline_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), ApiError> {
        Ok(self.decline_event_invite(user_id, event_id).await?)
    }

    async fn verify_invite_code(
        &self,
        data: PostInviteVerifyRequestBody,
    ) -> Result<PostInviteVerifyResponseBody, ApiError> {
        Ok(self.verify_invite_code(data).await?)
    }

    async fn get_sip_config(
        &self,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<SipConfigResource, ApiError> {
        Ok(self.get_sip_config(room_id_or_alias).await?)
    }

    async fn set_sip_config(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        modify_sip_config: PutSipConfigRequestBody,
    ) -> Result<(SipConfigResource, bool), ApiError> {
        Ok(self
            .set_sip_config(room_id_or_alias, modify_sip_config)
            .await?)
    }

    async fn delete_sip_config(&self, room_id_or_alias: RoomIdOrAlias) -> Result<(), ApiError> {
        Ok(self.delete_sip_config(room_id_or_alias).await?)
    }

    async fn get_streaming_targets(
        &self,
        user_id: UserId,
        room_id_or_alias: RoomIdOrAlias,
        pagination: &PagePaginationQuery,
    ) -> Result<GetRoomStreamingTargetsResponseBody, ApiError> {
        Ok(self
            .get_streaming_targets(user_id, room_id_or_alias, pagination)
            .await?)
    }

    async fn post_streaming_target(
        &self,
        current_user: RequestUser,
        room_id_or_alias: RoomIdOrAlias,
        query: StreamingTargetOptionsQuery,
        streaming_target: StreamingTarget,
    ) -> Result<PostRoomStreamingTargetResponseBody, ApiError> {
        Ok(self
            .post_streaming_target(current_user, room_id_or_alias, query, streaming_target)
            .await?)
    }

    async fn get_streaming_target(
        &self,
        user_id: UserId,
        path_params: RoomAndStreamingTargetId,
    ) -> Result<GetRoomStreamingTargetResponseBody, ApiError> {
        Ok(self.get_streaming_target(user_id, path_params).await?)
    }

    async fn patch_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
        streaming_target: PatchRoomStreamingTargetRequestBody,
    ) -> Result<PatchRoomStreamingTargetResponseBody, ApiError> {
        Ok(self
            .patch_streaming_target(current_user, path_params, query, streaming_target)
            .await?)
    }

    async fn delete_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_streaming_target(current_user, path_params, query)
            .await?)
    }

    async fn add_event_to_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<bool, ApiError> {
        Ok(self.add_event_to_favorites(current_user, event_id).await?)
    }

    async fn remove_event_from_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<(), ApiError> {
        Ok(self
            .remove_event_from_favorites(current_user, event_id)
            .await?)
    }

    async fn get_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<SharedFolder, ApiError> {
        Ok(self
            .get_shared_folder_for_event(current_user, event_id)
            .await?)
    }

    async fn put_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PutSharedFolderQuery,
    ) -> Result<(SharedFolder, bool), ApiError> {
        Ok(self
            .put_shared_folder_for_event(current_user, event_id, query)
            .await?)
    }

    async fn delete_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteSharedFolderQuery,
    ) -> Result<(), ApiError> {
        Ok(self
            .delete_shared_folder_for_event(current_user, event_id, query)
            .await?)
    }

    async fn patch_me(
        &self,
        current_user: RequestUser,
        patch: PatchMeRequestBody,
        access_token: &AccessToken,
    ) -> Result<Option<PrivateUserProfile>, ApiError> {
        Ok(self.patch_me(current_user, patch, access_token).await?)
    }

    async fn get_me(&self, current_user: RequestUser) -> Result<PrivateUserProfile, ApiError> {
        Ok(self.get_me(current_user).await?)
    }

    async fn get_my_tariff(&self, current_user: RequestUser) -> Result<TariffResource, ApiError> {
        Ok(self.get_my_tariff(current_user).await?)
    }

    async fn get_my_assets(
        &self,
        current_user: RequestUser,
        sorting: AssetSortingQuery,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetUserAssetsResponseBody, ItemCount), ApiError> {
        Ok(self
            .get_my_assets(current_user, sorting, pagination)
            .await?)
    }

    async fn get_user(
        &self,
        current_user: RequestUser,
        user_id: UserId,
    ) -> Result<PublicUserProfile, ApiError> {
        Ok(self.get_user(current_user, user_id).await?)
    }

    async fn find_users(
        &self,
        current_user: RequestUser,
        query: GetFindQuery,
    ) -> Result<GetFindResponseBody, ApiError> {
        Ok(self.find_users(current_user, query).await?)
    }

    async fn create_module_resource(
        &self,
        resource: NewModuleResource,
    ) -> Result<ModuleResource, ApiError> {
        Ok(self.create_module_resource(resource).await?)
    }

    async fn get_module_resources(
        &self,
        filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>, ApiError> {
        Ok(self.get_module_resources(filter).await?)
    }

    async fn patch_module_resources(
        &self,
        filter: ModuleResourceFilter,
        patch_operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>, ApiError> {
        Ok(self
            .patch_module_resources(filter, patch_operations)
            .await?)
    }

    async fn delete_module_resources(
        &self,
        filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>, ApiError> {
        Ok(self.delete_module_resources(filter).await?)
    }
}
