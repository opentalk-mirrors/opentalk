// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_utils::CaptureApiError;
use opentalk_inventory::UpdateRoomStreamingTarget;
use opentalk_types_api_v1::{
    error::ApiError,
    events::StreamingTargetOptionsQuery,
    pagination::PagePaginationQuery,
    rooms::{
        by_room_id::streaming_targets::{
            GetRoomStreamingTargetResponseBody, GetRoomStreamingTargetsResponseBody,
            PatchRoomStreamingTargetRequestBody, PatchRoomStreamingTargetResponseBody,
            PostRoomStreamingTargetResponseBody, RoomAndStreamingTargetId,
        },
        streaming_targets::UpdateStreamingTargetKind,
    },
};
use opentalk_types_common::{
    rooms::RoomIdOrAlias,
    streaming::{
        RoomStreamingTarget, RoomStreamingTargetResource, StreamingKind, StreamingTarget,
        StreamingTargetKind, StreamingTargetKindResource, StreamingTargetResource,
    },
    users::UserId,
};
use snafu::Report;

use crate::{
    ControllerBackend, controller_backend::utils::resolve_room_id,
    events::notifications::notify_event_invitees_by_room_about_update,
};

impl ControllerBackend {
    pub(crate) async fn get_streaming_targets(
        &self,
        user_id: UserId,
        room_id_or_alias: RoomIdOrAlias,
        _pagination: &PagePaginationQuery,
    ) -> Result<GetRoomStreamingTargetsResponseBody, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let room = inventory.get_room(&room_id_or_alias).await?;
        // TODO: No paginated DB access is done here for now though the API provides pagination parameters
        let room_streaming_targets = inventory.get_room_streaming_targets(room.id).await?;

        let with_streaming_key = room.created_by == user_id;
        let room_streaming_target_resources = room_streaming_targets
            .into_iter()
            .map(|rst| build_resource(rst, with_streaming_key))
            .collect();

        Ok(GetRoomStreamingTargetsResponseBody(
            room_streaming_target_resources,
        ))
    }

    pub(crate) async fn post_streaming_target(
        &self,
        current_user: RequestUser,
        room_id_or_alias: RoomIdOrAlias,
        query: StreamingTargetOptionsQuery,
        streaming_target: StreamingTarget,
    ) -> Result<PostRoomStreamingTargetResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();

        let room_id = resolve_room_id(inventory.as_mut(), &room_id_or_alias).await?;

        let room_streaming_target = inventory
            .create_room_streaming_target(room_id, streaming_target)
            .await?;

        if let Some(mail_service) = &mail_service {
            let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
            let current_user = inventory.get_user(current_user.id).await?;
            let room_tariff = self.get_room_tariff(room_id.into()).await?;

            notify_event_invitees_by_room_about_update(
                &self.user_search_client,
                &settings,
                mail_service,
                current_tenant,
                current_user,
                inventory.as_mut(),
                room_id,
                &room_tariff,
            )
            .await?;
        }

        Ok(PostRoomStreamingTargetResponseBody(room_streaming_target))
    }

    pub(crate) async fn get_streaming_target(
        &self,
        user_id: UserId,
        RoomAndStreamingTargetId {
            room_id_or_alias,
            streaming_target_id,
        }: RoomAndStreamingTargetId,
    ) -> Result<GetRoomStreamingTargetResponseBody, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let room = inventory.get_room(&room_id_or_alias).await?;

        let room_streaming_target = inventory
            .get_room_streaming_target_record(room.id, streaming_target_id)
            .await?;

        let with_streaming_key = room.created_by == user_id;

        let room_streaming_target = RoomStreamingTarget {
            id: room_streaming_target.id,
            streaming_target: StreamingTarget {
                name: room_streaming_target.name,
                kind: StreamingTargetKind::Custom {
                    streaming_endpoint: room_streaming_target.streaming_endpoint.parse().map_err(
                        |e| {
                            log::warn!(
                                "Invalid streaming endpoint url entry in db: {}",
                                Report::from_error(e)
                            );
                            ApiError::internal()
                        },
                    )?,
                    streaming_key: room_streaming_target.streaming_key,
                    public_url: room_streaming_target.public_url.parse().map_err(|e| {
                        log::warn!(
                            "Invalid streaming endpoint url entry in db: {}",
                            Report::from_error(e)
                        );
                        ApiError::internal()
                    })?,
                },
            },
        };

        let room_streaming_target_resource =
            build_resource(room_streaming_target, with_streaming_key);

        Ok(GetRoomStreamingTargetResponseBody(
            room_streaming_target_resource,
        ))
    }

    pub(crate) async fn patch_streaming_target(
        &self,
        current_user: RequestUser,
        RoomAndStreamingTargetId {
            room_id_or_alias,
            streaming_target_id,
        }: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
        streaming_target: PatchRoomStreamingTargetRequestBody,
    ) -> Result<PatchRoomStreamingTargetResponseBody, CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        if streaming_target.name.is_none() && streaming_target.kind.is_none() {
            return Err(ApiError::bad_request().into());
        }

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();

        let (kind, streaming_endpoint, streaming_key, public_url) = match streaming_target.kind {
            Some(kind) => match kind {
                UpdateStreamingTargetKind::Custom {
                    streaming_endpoint,
                    streaming_key,
                    public_url,
                } => (
                    Some(StreamingKind::Custom),
                    streaming_endpoint.map(|s| s.into()),
                    streaming_key.map(|s| s.into()),
                    public_url.map(|s| s.into()),
                ),
            },
            None => (None, None, None, None),
        };

        let room_streaming_target = inventory
            .update_room_streaming_target(
                &room_id_or_alias,
                streaming_target_id,
                UpdateRoomStreamingTarget {
                    name: streaming_target.name,
                    kind,
                    streaming_endpoint,
                    streaming_key,
                    public_url,
                },
            )
            .await?;
        let room_id = room_streaming_target.room_id;

        let kind = match room_streaming_target.kind {
            StreamingKind::Custom => StreamingTargetKind::Custom {
                streaming_endpoint: room_streaming_target.streaming_endpoint.parse().map_err(
                    |e| {
                        log::warn!(
                            "Invalid streaming endpoint url entry in db: {}",
                            Report::from_error(e)
                        );
                        ApiError::internal()
                    },
                )?,
                streaming_key: room_streaming_target.streaming_key,
                public_url: room_streaming_target.public_url.parse().map_err(|e| {
                    log::warn!("Invalid public url entry in db: {}", Report::from_error(e));
                    ApiError::internal()
                })?,
            },
        };

        let room_streaming_target = RoomStreamingTarget {
            id: room_streaming_target.id,
            streaming_target: StreamingTarget {
                name: room_streaming_target.name,
                kind,
            },
        };

        if let Some(mail_service) = &mail_service {
            let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
            let current_user = inventory.get_user(current_user.id).await?;
            let room_tariff = self.get_room_tariff(room_id.into()).await?;

            notify_event_invitees_by_room_about_update(
                &self.user_search_client,
                &settings,
                mail_service,
                current_tenant,
                current_user,
                inventory.as_mut(),
                room_id,
                &room_tariff,
            )
            .await?;
        }

        Ok(PatchRoomStreamingTargetResponseBody(room_streaming_target))
    }

    pub(crate) async fn delete_streaming_target(
        &self,
        current_user: RequestUser,
        RoomAndStreamingTargetId {
            room_id_or_alias,
            streaming_target_id,
        }: RoomAndStreamingTargetId,
        query: StreamingTargetOptionsQuery,
    ) -> Result<(), CaptureApiError> {
        let settings = self.settings_provider.get();
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let mail_service = (!query.suppress_email_notification)
            .then(|| self.mail_service.as_ref().clone())
            .flatten();

        inventory
            .delete_room_streaming_target(&room_id_or_alias, streaming_target_id)
            .await?;

        if let Some(mail_service) = &mail_service {
            let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
            let current_user = inventory.get_user(current_user.id).await?;
            let room_id = resolve_room_id(inventory.as_mut(), &room_id_or_alias).await?;
            let room_tariff = self.get_room_tariff(room_id.into()).await?;

            notify_event_invitees_by_room_about_update(
                &self.user_search_client,
                &settings,
                mail_service,
                current_tenant,
                current_user,
                inventory.as_mut(),
                room_id,
                &room_tariff,
            )
            .await?;
        }

        Ok(())
    }
}

/// Builds a streaming target resource
fn build_resource(
    room_streaming_target: RoomStreamingTarget,
    with_streaming_key: bool,
) -> RoomStreamingTargetResource {
    let kind_resource = match room_streaming_target.streaming_target.kind {
        StreamingTargetKind::Custom {
            streaming_endpoint,
            streaming_key,
            public_url,
        } => StreamingTargetKindResource::Custom {
            streaming_endpoint,
            streaming_key: if with_streaming_key {
                Some(streaming_key)
            } else {
                None
            },
            public_url,
        },
    };

    RoomStreamingTargetResource {
        id: room_streaming_target.id,
        streaming_target: StreamingTargetResource {
            name: room_streaming_target.streaming_target.name,
            kind: kind_resource,
        },
    }
}
