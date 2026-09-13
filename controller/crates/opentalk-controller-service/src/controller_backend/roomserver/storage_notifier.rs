// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! This module contains the implementation of a [`StorageNotifier`] which notifies the RoomServer about changes in the storage quota.

use std::num::NonZero;

use futures::{StreamExt as _, stream};
use opentalk_asset_storage::StorageNotifier;
use opentalk_roomserver_client::Client;
use opentalk_roomserver_room::RoomTaskRegistry;
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::users::UserId;

use crate::controller_backend::roomserver::websocket_adapter::WebSocketAdapter;

#[derive(Debug, Clone)]
/// A storage notifier for an external roomserver.
pub struct ExternalStorageNotifier {
    client: Client,
}

impl ExternalStorageNotifier {
    /// Creates a new [`ExternalStorageNotifier`]
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl StorageNotifier for ExternalStorageNotifier {
    async fn notify(&self, user_id: UserId, _old_quota: Quota, new_quota: Quota) {
        if let Err(err) = self.client.post_storage_quota(user_id, new_quota).await {
            log::error!("Failed to post roomserver storage quota: {err}");
        } else {
            log::debug!("Notified RoomServer about quota change");
        }
    }
}

#[derive(Debug)]
/// A storage notifier for an embedded roomserver.
pub struct InternalStorageNotifier {
    room_tasks: RoomTaskRegistry<WebSocketAdapter>,
    parallel_requests: NonZero<usize>,
}

impl InternalStorageNotifier {
    pub fn new(
        room_tasks: RoomTaskRegistry<WebSocketAdapter>,
        parallel_requests: NonZero<usize>,
    ) -> Self {
        Self {
            room_tasks,
            parallel_requests,
        }
    }
}

#[async_trait::async_trait]
impl StorageNotifier for InternalStorageNotifier {
    async fn notify(&self, user_id: UserId, _old_quota: Quota, new_quota: Quota) {
        let handles = self.room_tasks.task_handles_by_creator(user_id).await;

        let parallel_requests = self.parallel_requests.get();
        stream::iter(handles)
            .map(|(room_id, handle)| {
                let quota = new_quota.clone();
                async move {
                    _ = handle.set_storage_quota(quota).await.inspect_err(|err| {
                        log::warn!("Failed to set storage quota for room {room_id}: {err}");
                    });
                }
            })
            .for_each_concurrent(parallel_requests, |fut| fut)
            .await;
    }
}
