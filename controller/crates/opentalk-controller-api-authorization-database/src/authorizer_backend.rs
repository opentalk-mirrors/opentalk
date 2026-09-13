// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use async_trait::async_trait;
use opentalk_controller_api_authorization::authorization::{
    Admission, AuthorizationChange, AuthorizationChangeError, AuthorizationTarget,
    AuthorizerBackend, Resource,
};
use opentalk_controller_settings::SettingsProvider;
use opentalk_inventory::InventoryProvider;
use opentalk_types_common::{features::FeatureId, modules::ModuleId};

use crate::Result;

/// The [`AuthorizerBackend`] for OpenTalk
#[derive(Debug, Clone)]
pub struct OpenTalkAuthorizerBackend {
    pub(crate) inventory: Arc<dyn InventoryProvider>,
    pub(crate) settings: SettingsProvider,
    pub(crate) module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
}

#[async_trait]
impl AuthorizerBackend for OpenTalkAuthorizerBackend {
    async fn authorize(
        &self,
        AuthorizationTarget {
            resource,
            authenticated_subjects: subjects,
            access_method: method,
        }: AuthorizationTarget,
    ) -> Result<Admission> {
        match resource {
            Resource::AuthLogin => Ok(Self::authorize_auth_login()),
            Resource::AuthLogout => Ok(Self::authorize_auth_logout()),
            Resource::Turn => Ok(Self::authorize_turn()),
            Resource::Events => Ok(self.authorize_events(subjects)),
            Resource::EventsInstances => Ok(self.authorize_events_instances(subjects, method)),
            Resource::Event(event_id) => self.authorize_event(subjects, method, event_id).await,
            Resource::EventInvites(event_id) => {
                self.authorize_event_invites(subjects, method, event_id)
                    .await
            }
            Resource::EventEmailInvite(event_id) => {
                self.authorize_event_email_invite(subjects, method, event_id)
                    .await
            }
            Resource::EventUserInvite(event_id, _) => {
                self.authorize_event_user_invite(subjects, method, event_id)
                    .await
            }
            Resource::EventInvite(event_id) => {
                self.authorize_event_invite(subjects, method, event_id)
                    .await
            }
            Resource::EventInstances(event_id) => {
                self.authorize_event_instances(subjects, method, event_id)
                    .await
            }
            Resource::EventSharedFolder(event_id) => {
                self.authorize_event_shared_folder(subjects, method, event_id)
                    .await
            }
            Resource::EventInstance(event_id, _) => {
                self.authorize_event_instance(subjects, method, event_id)
                    .await
            }
            Resource::Rooms => Ok(self.authorize_rooms(subjects).await),
            Resource::Room(room_id) => self.authorize_room(subjects, method, &room_id).await,
            Resource::RoomEvent(room_id_or_alias) => {
                self.authorize_room_event(subjects, method, &room_id_or_alias)
                    .await
            }
            Resource::RoomNameVerify => Ok(Self::authorize_room_name_verify(subjects)),
            Resource::RoomAssets(room_id) => {
                self.authorize_room_assets(subjects, method, &room_id).await
            }
            Resource::RoomAsset(room_id, _) => {
                self.authorize_room_asset(subjects, method, &room_id).await
            }
            Resource::RoomAssetDownload(room_id_or_alias, _) => {
                self.authorize_room_asset_download(subjects, method, room_id_or_alias)
                    .await
            }
            Resource::RoomStreamingTargets(room_id) => {
                self.authorize_room_streaming_targets(subjects, method, &room_id)
                    .await
            }
            Resource::RoomStreamingTarget(room_id, _) => {
                self.authorize_room_streaming_target(subjects, method, &room_id)
                    .await
            }
            Resource::RoomSip(room_id) => self.authorize_room_sip(subjects, method, &room_id).await,
            Resource::RoomStart(room_id_or_alias) => {
                self.authorize_room_start(subjects, method, &room_id_or_alias)
                    .await
            }
            // Removed endpoints: the handlers redirect or respond with `410 Gone` themselves.
            Resource::RoomStartInvited(_) | Resource::RemovedRoomInvites => Ok(Admission::Allowed),
            Resource::InviteVerify => Ok(Self::authorize_invite_verify()),
            Resource::Signaling(_) => Ok(Self::authorize_signaling()),
            Resource::RoomAssetDownloadProxy(_, _) => {
                Ok(Self::authorize_room_asset_download_proxy())
            }
            Resource::RoomTariff(room_id) => {
                self.authorize_room_tariff(subjects, method, &room_id).await
            }
            Resource::UserFind => Ok(self.authorize_user_find(subjects, method)),
            Resource::UserMe => Ok(self.authorize_user_me(subjects)),
            Resource::UserMeAssets => Ok(self.authorize_user_me_assets(subjects, method)),
            Resource::UserMeEventFavorite(event_id) => {
                self.authorize_user_me_event_favorite(subjects, method, event_id)
                    .await
            }
            Resource::UserMePendingInvites => {
                Ok(self.authorize_user_me_pending_invites(subjects, method))
            }
            Resource::UserMeTariff => Ok(self.authorize_user_me_tariff(subjects, method)),
            Resource::UserProfile(_) => Ok(self.authorize_user_profile(subjects, method)),
        }
    }

    async fn apply_changes(
        &self,
        _changeset: &[AuthorizationChange],
    ) -> Result<(), AuthorizationChangeError> {
        // This implementation should remain empty, because we don't need to update or invalidate
        // any cache on change, we always retrieve the permissions from the inventory.
        Ok(())
    }
}

impl OpenTalkAuthorizerBackend {
    /// Create a new [`OpenTalkAuthorizerBackend`].
    pub const fn new(
        inventory: Arc<dyn InventoryProvider>,
        settings: SettingsProvider,
        module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
    ) -> Self {
        Self {
            inventory,
            settings,
            module_features,
        }
    }
}
