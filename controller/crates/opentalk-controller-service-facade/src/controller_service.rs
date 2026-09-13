// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use openidconnect::AccessToken;

mod asset_download_proxy_stream;
mod start_room_error;

pub use asset_download_proxy_stream::AssetDownloadProxyStream;
use async_trait::async_trait;
use bytes::Bytes;
use futures_core::Stream;
use opentalk_asset_storage::{
    AssetSaved, ByStreamExt, NewAssetFileName, ObjectStorageError, StorageNotifier,
};
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
        GetLoginResponseBody, LogoutToken, PostLoginResponseBody, login::AuthLoginPostRequestBody,
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
    modules::ModuleId,
    pagination::{ItemCount, Page, PageSize},
    rooms::{GuestAccess, RoomIdOrAlias, RoomName, RoomPassword},
    shared_folders::SharedFolder,
    streaming::StreamingTarget,
    tariffs::TariffResource,
    users::UserId,
};
pub use start_room_error::StartRoomError;
use url::Url;

use crate::RequestUser;

/// Trait implemented by OpenTalk controller service backends
#[async_trait(?Send)]
pub trait OpenTalkControllerService: Send + Sync {
    /// Get the configured OIDC provider
    async fn get_login(&self) -> GetLoginResponseBody;

    /// Triggers OIDC back channel logout
    async fn post_logout(&self, logout_token: &LogoutToken) -> Result<(), ApiError>;

    /// Post a login request.
    async fn post_login(
        &self,
        body: AuthLoginPostRequestBody,
    ) -> Result<PostLoginResponseBody, ApiError>;

    /// Get all accessible rooms
    async fn get_rooms(
        &self,
        current_user_id: UserId,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetRoomsResponseBody, ItemCount), ApiError>;

    /// Create a new room
    #[allow(clippy::too_many_arguments)]
    async fn create_room(
        &self,
        current_user: RequestUser,
        name: Option<RoomName>,
        password: Option<RoomPassword>,
        enable_sip: bool,
        waiting_room: bool,
        guest_access: Option<GuestAccess>,
        e2e_encryption: bool,
    ) -> Result<RoomResource, ApiError>;

    /// Patch a room with the provided fields
    async fn patch_room(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        name: Option<Option<RoomName>>,
        password: Option<Option<RoomPassword>>,
        waiting_room: Option<bool>,
        guest_access: Option<GuestAccess>,
        e2e_encryption: Option<bool>,
    ) -> Result<RoomResource, ApiError>;

    /// Delete a room and its owned resources.
    async fn delete_room(
        &self,
        current_user: RequestUser,
        room_id_or_alias: RoomIdOrAlias,
        force_delete_reference_if_external_services_fail: bool,
    ) -> Result<(), ApiError>;

    /// Verify if a room name is valid and available.
    async fn verify_room_name(
        &self,
        name: RoomName,
    ) -> Result<PostRoomNameVerifyResponseBody, ApiError>;

    /// Get a room
    async fn get_room(&self, room_id_or_alias: RoomIdOrAlias) -> Result<RoomResource, ApiError>;

    /// Get a room's tariff
    async fn get_room_tariff(
        &self,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<TariffResource, ApiError>;

    /// Get a room's event
    async fn get_room_event(
        &self,
        current_user: Option<RequestUser>,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<GetRoomEventResponseBody, ApiError>;

    /// Start a roomserver signaling session as a registered user
    async fn start_room_session(
        &self,
        current_user: Option<RequestUser>,
        room_id_or_alias: RoomIdOrAlias,
        request: PostRoomsRoomserverStartRequestBody,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, ApiError>;

    /// Starts a signaling session for recording
    async fn start_recording(
        &self,
        body: RecordingTarget,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, ApiError>;

    /// Starts a signaling session for transcription
    async fn start_transcription(
        &self,
        body: RecordingTarget,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, ApiError>;

    /// Starts a signaling session for call-in
    async fn start_call_in(
        &self,
        request: PostCallInStartRoomServerRequestBody,
        host: Url,
    ) -> Result<RoomserverStartResponseBody, ApiError>;

    /// Get the assets associated with a room.
    async fn get_room_assets(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        pagination: &PagePaginationQuery,
    ) -> Result<(RoomsByRoomIdAssetsGetResponseBody, ItemCount), ApiError>;

    /// Get a specific asset inside a room.
    async fn get_room_asset(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<ByStreamExt, ApiError>;

    /// Get a short-lived download URL for a specific asset.
    async fn get_room_asset_proxy_download_token(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<String, ApiError>;

    /// Get a proxied asset data stream
    async fn get_asset_proxy_download_stream(
        &self,
        asset_id: AssetId,
        token: String,
        range_header: Option<String>,
    ) -> Result<AssetDownloadProxyStream, ApiError>;

    /// Create an asset for a room from an uploaded file.
    async fn create_room_asset(
        &self,
        storage_notifier: &dyn StorageNotifier,
        room_id_or_alias: RoomIdOrAlias,
        filename: NewAssetFileName,
        namespace: Option<ModuleId>,
        data: Box<dyn Stream<Item = Result<Bytes, ObjectStorageError>> + Unpin>,
    ) -> Result<(AssetResource, AssetSaved), ApiError>;

    /// Delete an asset from a room.
    async fn delete_room_asset(
        &self,
        storage_notifier: &dyn StorageNotifier,
        room_id_or_alias: RoomIdOrAlias,
        asset_id: AssetId,
    ) -> Result<(), ApiError>;

    /// Create a new event
    async fn new_event(
        &self,
        current_user: RequestUser,
        event: PostEventsBody,
        query: EventOptionsQuery,
    ) -> Result<EventResource, ApiError>;

    /// Get a list of events and exceptions
    async fn get_events_and_exceptions_interwoven(
        &self,
        current_user: RequestUser,
        query: GetEventsQuery,
    ) -> Result<(Vec<EventOrException>, Option<String>, Option<String>), ApiError>;

    /// Get an event
    async fn get_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: GetEventQuery,
    ) -> Result<EventResource, ApiError>;

    /// Patch an event
    async fn patch_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PatchEventQuery,
        patch: PatchEventBody,
    ) -> Result<Option<EventResource>, ApiError>;

    /// Delete an event and its owned resources, including the associated room.
    async fn delete_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteEventsQuery,
    ) -> Result<(), ApiError>;

    /// Get a list of events and instances
    async fn get_events_and_instances_interwoven(
        &self,
        current_user: RequestUser,
        query: GetEventsAndInstancesQuery,
    ) -> Result<(Vec<EventOrInstance>, Option<String>, Option<String>), ApiError>;

    /// Get a list of the instances of an event
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
    >;

    /// Get an event instance
    async fn get_event_instance(
        &self,
        current_user: &RequestUser,
        path: EventInstancePath,
        query: EventInstanceQuery,
    ) -> Result<GetEventInstanceResponseBody, ApiError>;

    /// Modifies an event instance
    async fn patch_event_instance(
        &self,
        current_user: RequestUser,
        path: EventInstancePath,
        query: EventInstanceQuery,
        patch: PatchEventInstanceBody,
    ) -> Result<Option<EventInstance>, ApiError>;

    /// Get the invites for an event
    async fn get_invites_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: GetEventsInvitesQuery,
    ) -> Result<(Vec<EventInvitee>, PageSize, Page, ItemCount), ApiError>;

    /// Create a new invite to an event
    async fn create_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PostEventInviteQuery,
        create_invite: PostEventInviteBody,
    ) -> Result<bool, ApiError>;

    /// Patch an event invite with the provided fields
    async fn update_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        user_id: UserId,
        update_invite: &PatchInviteBody,
    ) -> Result<(), ApiError>;

    /// Patch an event email invite with the provided fields
    async fn update_email_invite_to_event(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        update_invite: &PatchEmailInviteBody,
    ) -> Result<(), ApiError>;

    /// Delete an invite from an event
    async fn delete_invite_to_event(
        &self,
        current_user: RequestUser,
        path: DeleteEventInvitePath,
        query: EventOptionsQuery,
    ) -> Result<(), ApiError>;

    /// Delete an invite from an event
    async fn delete_email_invite_to_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        email: EmailAddress,
        query: EventOptionsQuery,
    ) -> Result<(), ApiError>;

    /// Get information about pending invites
    async fn get_event_invites_pending(
        &self,
        user_id: UserId,
    ) -> Result<GetEventInvitesPendingResponseBody, ApiError>;

    /// Accept an invite to an event
    async fn accept_event_invite(&self, user_id: UserId, event_id: EventId)
    -> Result<(), ApiError>;

    /// Decline an invite to an event
    async fn decline_event_invite(
        &self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(), ApiError>;

    /// Verify an invite code
    async fn verify_invite_code(
        &self,
        data: PostInviteVerifyRequestBody,
    ) -> Result<PostInviteVerifyResponseBody, ApiError>;

    /// Get the sip config for the specified room.
    async fn get_sip_config(
        &self,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<SipConfigResource, ApiError>;

    /// Modify the sip configuration of a room. A new sip configuration is created
    /// if none was set before.
    async fn set_sip_config(
        &self,
        room_id_or_alias: RoomIdOrAlias,
        modify_sip_config: PutSipConfigRequestBody,
    ) -> Result<(SipConfigResource, bool), ApiError>;

    /// Delete the SIP configuration of a room.
    async fn delete_sip_config(&self, room_id_or_alias: RoomIdOrAlias) -> Result<(), ApiError>;

    /// Lists the streaming targets of a room
    async fn get_streaming_targets(
        &self,
        user_id: UserId,
        room_id_or_alias: RoomIdOrAlias,
        pagination: &PagePaginationQuery,
    ) -> Result<GetRoomStreamingTargetsResponseBody, ApiError>;

    /// Creates a new streaming target
    async fn post_streaming_target(
        &self,
        current_user: RequestUser,
        room_id_or_alias: RoomIdOrAlias,
        query: StreamingTargetOptionsQuery,
        streaming_target: StreamingTarget,
    ) -> Result<PostRoomStreamingTargetResponseBody, ApiError>;

    /// Gets a streaming target
    async fn get_streaming_target(
        &self,
        user_id: UserId,
        path_params: RoomAndStreamingTargetId,
    ) -> Result<GetRoomStreamingTargetResponseBody, ApiError>;

    /// Updates a streaming target
    async fn patch_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
        update_streaming_target: PatchRoomStreamingTargetRequestBody,
    ) -> Result<PatchRoomStreamingTargetResponseBody, ApiError>;

    /// Deletes a streaming target
    async fn delete_streaming_target(
        &self,
        current_user: RequestUser,
        path_params: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
    ) -> Result<(), ApiError>;

    /// Add an event to the current user's favorites
    async fn add_event_to_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<bool, ApiError>;

    /// Remove an event from the current user's favorites
    async fn remove_event_from_favorites(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<(), ApiError>;

    /// Get the shared folder for an event
    async fn get_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
    ) -> Result<SharedFolder, ApiError>;

    /// Create a shared folder for an event
    async fn put_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: PutSharedFolderQuery,
    ) -> Result<(SharedFolder, bool), ApiError>;

    /// Delete the shared folder of an event
    async fn delete_shared_folder_for_event(
        &self,
        current_user: RequestUser,
        event_id: EventId,
        query: DeleteSharedFolderQuery,
    ) -> Result<(), ApiError>;

    /// Patch the current user's profile.
    async fn patch_me(
        &self,
        current_user: RequestUser,
        patch: PatchMeRequestBody,
        access_token: &AccessToken,
    ) -> Result<Option<PrivateUserProfile>, ApiError>;

    /// Get the current user's profile.
    async fn get_me(&self, current_user: RequestUser) -> Result<PrivateUserProfile, ApiError>;

    /// Get the current user tariff information.
    async fn get_my_tariff(&self, current_user: RequestUser) -> Result<TariffResource, ApiError>;

    /// Get the assets associated with the user.
    async fn get_my_assets(
        &self,
        current_user: RequestUser,
        sorting: AssetSortingQuery,
        pagination: &PagePaginationQuery,
    ) -> Result<(GetUserAssetsResponseBody, ItemCount), ApiError>;

    /// Get a user's public profile.
    async fn get_user(
        &self,
        current_user: RequestUser,
        user_id: UserId,
    ) -> Result<PublicUserProfile, ApiError>;

    /// Find users.
    async fn find_users(
        &self,
        current_user: RequestUser,
        query: GetFindQuery,
    ) -> Result<GetFindResponseBody, ApiError>;

    /// Create a new module resource
    async fn create_module_resource(
        &self,
        resource: NewModuleResource,
    ) -> Result<ModuleResource, ApiError>;

    /// Get module resources matching the filter
    async fn get_module_resources(
        &self,
        filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>, ApiError>;

    /// Patch the module resources matching the filter
    async fn patch_module_resources(
        &self,
        filter: ModuleResourceFilter,
        patch_operations: Vec<ModuleResourceOperation>,
    ) -> Result<Vec<ModuleResource>, ApiError>;

    /// Delete the module resources matching the filter
    async fn delete_module_resources(
        &self,
        filter: ModuleResourceFilter,
    ) -> Result<Vec<ModuleResource>, ApiError>;
}
