// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_api_v1::events::InstanceId;
use opentalk_types_common::{
    assets::AssetId, events::EventId, rooms::RoomIdOrAlias, roomserver::Token,
    streaming::StreamingTargetId, users::UserId,
};

/// Specification of a resource provided by the OpenTalk Controller API.
///
/// Variants come in two flavours:
///
/// * **Authenticated resources** require the caller to be a [`Subject`] (i.e.
///   an authenticated or unauthenticated user) and are checked
///   against an ACL by the authorization backend.
/// * **Public resources** are served without authentication and the
///   authorization backend admits any caller — including unauthenticated
///   ones. They are still represented here so that the middleware can map
///   every routed request to a known [`Resource`] and the "allowed by
///   default" decision is explicit and testable.
///
/// [`Subject`]: super::Subject
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Resource {
    /// The OIDC login endpoint.
    ///
    /// Served under `/v1/auth/login`. Public: callers are unauthenticated
    /// at this point and the endpoint just exchanges an OIDC ID token for
    /// the controller's own representation of the user.
    AuthLogin,

    /// The OIDC back-channel logout endpoint.
    ///
    /// Served under `/v1/auth/logout`. Public: the logout token in the
    /// request body is validated by the endpoint itself.
    AuthLogout,

    /// The (deprecated) TURN credentials endpoint.
    ///
    /// Served under `/v1/turn`. Public: the endpoint always returns an
    /// empty response and only exists for backwards compatibility.
    Turn,

    /// The events list resource.
    ///
    /// Served under `/v1/events`.
    Events,

    /// The event instances list resource.
    ///
    /// Served under `/v1/events/instances`.
    EventsInstances,

    /// An event resource.
    ///
    /// Served under `/v1/events/{event_id}`.
    Event(EventId),

    /// The list of invites for an event resource.
    ///
    /// Served under `/v1/events/{event_id}/invites`.
    EventInvites(EventId),

    /// An event e-mail invite resource.
    ///
    /// Served under `/v1/events/{event_id}/invites/email`.
    EventEmailInvite(EventId),

    /// An event user invite resource.
    ///
    /// Served under `/v1/events/{event_id}/invites/{user_id}`.
    EventUserInvite(EventId, UserId),

    /// An event invite resource.
    ///
    /// Served under `/v1/events/{event_id}/invite`.
    EventInvite(EventId),

    /// The instances of an event.
    ///
    /// Served under `/v1/events/{event_id}/instances`.
    EventInstances(EventId),

    /// The shared folder of an event.
    ///
    /// Served under `/v1/events/{event_id}/shared_folder`.
    EventSharedFolder(EventId),

    /// An event instance resource.
    ///
    /// Served under `/v1/events/{event_id}/instances/{instance_id}`.
    EventInstance(EventId, InstanceId),

    /// The rooms list resource.
    ///
    /// Served under `/v1/rooms`.
    Rooms,

    /// A room resource.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}`.
    Room(RoomIdOrAlias),

    /// A room event resource.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/event`.
    RoomEvent(RoomIdOrAlias),

    /// The removed room invite-management endpoints.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/invites` and
    /// `/v1/rooms/{room_id_or_alias}/invites/{invite_code}`. Public: the invite-code API
    /// has been removed and these endpoints respond with `410 Gone` without
    /// exposing any room data.
    RemovedRoomInvites,

    /// The list of assets for a room.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/assets`.
    RoomAssets(RoomIdOrAlias),

    /// An asset stored with a room.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/assets/{asset_id}`.
    RoomAsset(RoomIdOrAlias, AssetId),

    /// An asset download for a room.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/assets/{asset_id}/download`.
    RoomAssetDownload(RoomIdOrAlias, AssetId),

    /// The list of streaming targets for a room.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/streaming_targets`.
    RoomStreamingTargets(RoomIdOrAlias),

    /// A streaming target for a room.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/streaming_targets/{streaming_target_id}`.
    RoomStreamingTarget(RoomIdOrAlias, StreamingTargetId),

    /// The sip config for a room.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/sip`.
    RoomSip(RoomIdOrAlias),

    /// The room start endpoint.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/start`.
    RoomStart(RoomIdOrAlias),
    /// The removed invite-based room start endpoint.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/start_invited` and
    /// `/v1/rooms/{room_id_or_alias}/roomserver/start_invited`. Public: the endpoint
    /// issues a permanent redirect to [`Self::RoomStart`], which performs the
    /// actual authorization.
    RoomStartInvited(RoomIdOrAlias),

    /// The room name verification endpoint.
    ///
    /// Served under `v1/rooms/name/verify`.
    RoomNameVerify,

    /// The invite verification endpoint.
    ///
    /// Served under `/v1/invite/verify`. Public: the endpoint validates
    /// the invite code carried in the request body itself.
    InviteVerify,

    /// The signaling WebSocket endpoint.
    ///
    /// Served under `/v1/signaling/{token}`. Public: access is gated by
    /// the signaling token in the URL, which the endpoint validates
    /// itself before upgrading the WebSocket.
    Signaling(Token),

    /// The asset download proxy endpoint.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/assets/{asset_id}/proxy`.
    /// Public: access is gated by the signed download token supplied in
    /// the query string, which the endpoint validates itself.
    RoomAssetDownloadProxy(RoomIdOrAlias, AssetId),

    /// The room tariff endpoint.
    ///
    /// Served under `/v1/rooms/{room_id_or_alias}/tariff`.
    RoomTariff(RoomIdOrAlias),

    /// The user search.
    ///
    /// Served under `/v1/users/find`.
    UserFind,

    /// The user profile information.
    ///
    /// Served under `/v1/users/me`.
    UserMe,

    /// The user assets.
    ///
    /// Served under `/v1/users/me/assets`.
    UserMeAssets,

    /// The user event favorite.
    ///
    /// Served under `/v1/users/me/event_favorites/{user_id}`.
    UserMeEventFavorite(EventId),

    /// The pending invites for a user.
    ///
    /// Served under `/v1/users/me/pending_invites`.
    UserMePendingInvites,

    /// The user profile tariff information.
    ///
    /// Served under `/v1/users/me/tariff`.
    UserMeTariff,

    /// The public profile of a user.
    ///
    /// Served under `/v1/users/{user_id}`.
    UserProfile(UserId),
}

#[cfg(feature = "actix-web")]
pub(super) mod actix_web_impls {
    use actix_router::{Path, PathDeserializer, ResourceDef};
    use actix_web::{dev::ServiceRequest, error::PathError};
    use opentalk_types_common::rooms::RoomIdOrAlias;
    use serde::de::DeserializeOwned;
    use snafu::{ResultExt, Snafu, ensure};

    use super::*;

    #[derive(Debug, Snafu)]
    pub enum TryFromResourceError {
        #[snafu(display("Unknown resource path {path:?}"))]
        UnknownResourcePath { path: String },

        #[snafu(display("Unknown resource path pattern {pattern:?}"))]
        UnknownResourcePathPattern { pattern: String },

        #[snafu(display(
            "Failed to extract path parameters for resource path pattern {pattern:?}: {source}"
        ))]
        InvalidResourcePathPattern { source: PathError, pattern: String },
    }

    impl TryFrom<&ServiceRequest> for Resource {
        type Error = TryFromResourceError;

        fn try_from(req: &ServiceRequest) -> Result<Self, Self::Error> {
            ensure!(
                req.resource_map().has_resource(req.path()),
                UnknownResourcePathSnafu {
                    path: req.path().to_string()
                }
            );
            let Some(pattern) = req.resource_map().match_pattern(req.path()) else {
                return UnknownResourcePathSnafu {
                    path: req.path().to_string(),
                }
                .fail()?;
            };

            match pattern.as_str() {
                "/v1/auth/login" => Ok(Resource::AuthLogin),
                "/v1/auth/logout" => Ok(Resource::AuthLogout),
                "/v1/turn" => Ok(Resource::Turn),
                "/v1/events" => Ok(Resource::Events),
                "/v1/events/instances" => Ok(Resource::EventsInstances),
                "/v1/events/{event_id}" => {
                    let event_id = extract_path::<EventId>(req.path(), pattern)?;
                    Ok(Resource::Event(event_id))
                }
                "/v1/events/{event_id}/invites" => {
                    let event_id = extract_path::<EventId>(req.path(), pattern)?;
                    Ok(Resource::EventInvites(event_id))
                }
                "/v1/events/{event_id}/invites/email" => {
                    let event_id = extract_path::<EventId>(req.path(), pattern)?;
                    Ok(Resource::EventEmailInvite(event_id))
                }
                "/v1/events/{event_id}/invites/{user_id}" => {
                    let (event_id, user_id) =
                        extract_path::<(EventId, UserId)>(req.path(), pattern)?;
                    Ok(Resource::EventUserInvite(event_id, user_id))
                }
                "/v1/events/{event_id}/invite" => {
                    let event_id = extract_path::<EventId>(req.path(), pattern)?;
                    Ok(Resource::EventInvite(event_id))
                }
                "/v1/events/{event_id}/instances" => {
                    let event_id = extract_path::<EventId>(req.path(), pattern)?;
                    Ok(Resource::EventInstances(event_id))
                }
                "/v1/events/{event_id}/instances/{instance_id}" => {
                    let (event_id, instance_id) =
                        extract_path::<(EventId, InstanceId)>(req.path(), pattern)?;
                    Ok(Resource::EventInstance(event_id, instance_id))
                }
                "/v1/events/{event_id}/shared_folder" => {
                    let event_id = extract_path::<EventId>(req.path(), pattern)?;
                    Ok(Resource::EventSharedFolder(event_id))
                }
                "/v1/rooms" => Ok(Resource::Rooms),
                "/v1/rooms/{room_id_or_alias}" => {
                    let room_id_or_alias = extract_path::<RoomIdOrAlias>(req.path(), pattern)?;
                    Ok(Resource::Room(room_id_or_alias))
                }
                "/v1/rooms/{room_id_or_alias}/event" => {
                    let room_id_or_alias = extract_path::<RoomIdOrAlias>(req.path(), pattern)?;
                    Ok(Resource::RoomEvent(room_id_or_alias))
                }
                "/v1/rooms/{room_id_or_alias}/invites"
                | "/v1/rooms/{room_id_or_alias}/invites/{invite_code}" => {
                    Ok(Resource::RemovedRoomInvites)
                }
                "/v1/rooms/{room_id_or_alias}/assets" => {
                    let room_id_or_alias = extract_path::<RoomIdOrAlias>(req.path(), pattern)?;
                    Ok(Resource::RoomAssets(room_id_or_alias))
                }
                "/v1/rooms/{room_id_or_alias}/assets/{asset_id}" => {
                    let (room_id_or_alias, asset_id) =
                        extract_path::<(RoomIdOrAlias, AssetId)>(req.path(), pattern)?;
                    Ok(Resource::RoomAsset(room_id_or_alias, asset_id))
                }
                "/v1/rooms/{room_id_or_alias}/assets/{asset_id}/download" => {
                    let (room_id_or_alias, asset_id) =
                        extract_path::<(RoomIdOrAlias, AssetId)>(req.path(), pattern)?;
                    Ok(Resource::RoomAssetDownload(room_id_or_alias, asset_id))
                }
                "/v1/rooms/{room_id_or_alias}/assets/{asset_id}/proxy" => {
                    let (room_id_or_alias, asset_id) =
                        extract_path::<(RoomIdOrAlias, AssetId)>(req.path(), pattern)?;
                    Ok(Resource::RoomAssetDownloadProxy(room_id_or_alias, asset_id))
                }
                "/v1/rooms/{room_id_or_alias}/sip" => {
                    let room_id_or_alias = extract_path::<RoomIdOrAlias>(req.path(), pattern)?;
                    Ok(Resource::RoomSip(room_id_or_alias))
                }
                "/v1/rooms/{room_id_or_alias}/streaming_targets" => {
                    let room_id_or_alias = extract_path::<RoomIdOrAlias>(req.path(), pattern)?;
                    Ok(Resource::RoomStreamingTargets(room_id_or_alias))
                }
                "/v1/rooms/{room_id_or_alias}/streaming_targets/{streaming_target_id}" => {
                    let (room_id_or_alias, streaming_target_id) =
                        extract_path::<(RoomIdOrAlias, StreamingTargetId)>(req.path(), pattern)?;
                    Ok(Resource::RoomStreamingTarget(
                        room_id_or_alias,
                        streaming_target_id,
                    ))
                }
                "/v1/rooms/{room_id_or_alias}/start"
                | "/v1/rooms/{room_id_or_alias}/roomserver/start" => {
                    let room_id_or_alias = extract_path::<RoomIdOrAlias>(req.path(), pattern)?;
                    Ok(Resource::RoomStart(room_id_or_alias))
                }
                "/v1/rooms/{room_id_or_alias}/start_invited"
                | "/v1/rooms/{room_id_or_alias}/roomserver/start_invited" => {
                    let room_id_or_alias = extract_path::<RoomIdOrAlias>(req.path(), pattern)?;
                    Ok(Resource::RoomStartInvited(room_id_or_alias))
                }
                "/v1/rooms/name/verify" => Ok(Resource::RoomNameVerify),
                "/v1/invite/verify" => Ok(Resource::InviteVerify),
                "/v1/signaling/{token}" => {
                    let token = extract_path::<Token>(req.path(), pattern)?;
                    Ok(Resource::Signaling(token))
                }
                "/v1/rooms/{room_id_or_alias}/tariff" => {
                    let room_id_or_alias = extract_path::<RoomIdOrAlias>(req.path(), pattern)?;
                    Ok(Resource::RoomTariff(room_id_or_alias))
                }
                "/v1/users/find" => Ok(Resource::UserFind),
                "/v1/users/me" => Ok(Resource::UserMe),
                "/v1/users/me/assets" => Ok(Resource::UserMeAssets),
                "/v1/users/me/event_favorites/{event_id}" => {
                    let event_id = extract_path::<EventId>(req.path(), pattern)?;
                    Ok(Resource::UserMeEventFavorite(event_id))
                }
                "/v1/users/me/pending_invites" => Ok(Resource::UserMePendingInvites),
                "/v1/users/me/tariff" => Ok(Resource::UserMeTariff),
                "/v1/users/{user_id}" => {
                    let user_id = extract_path::<UserId>(req.path(), pattern)?;
                    Ok(Resource::UserProfile(user_id))
                }
                pattern => UnknownResourcePathPatternSnafu {
                    pattern: pattern.to_string(),
                }
                .fail(),
            }
        }
    }

    fn extract_path<T: DeserializeOwned>(
        path: &str,
        pattern: String,
    ) -> Result<T, TryFromResourceError> {
        let resource = ResourceDef::prefix(&pattern);
        let mut path = Path::new(path);

        let matches = resource.capture_match_info(&mut path);
        if !matches {
            tracing::warn!("Path {path:?} didn't match pattern {pattern:?}");
        }

        let path: T = serde::de::Deserialize::deserialize(PathDeserializer::new(&path))
            .map_err(move |err| {
                tracing::debug!(
                    "Failed during Path extractor deserialization. \
                         Request path: {path:?}",
                );

                PathError::Deserialize(err)
            })
            .context(InvalidResourcePathPatternSnafu { pattern })?;
        Ok(path)
    }
}
