// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::fmt::Debug;

use async_trait::async_trait;
use opentalk_types_api_internal::module_assets::Quota;
use opentalk_types_common::users::UserId;

#[async_trait]
pub trait StorageNotifier: Send + Sync + Debug {
    async fn notify(&self, user_id: UserId, old_quota: Quota, new_quota: Quota);
}

#[async_trait]
impl<T> StorageNotifier for Option<T>
where
    T: StorageNotifier + Sync,
{
    async fn notify(&self, user_id: UserId, old_quota: Quota, new_quota: Quota) {
        if let Some(notifier) = self {
            notifier.notify(user_id, old_quota, new_quota).await;
        }
    }
}

#[derive(Debug)]
pub struct NoOpStorageNotifier;

#[async_trait]
impl StorageNotifier for NoOpStorageNotifier {
    async fn notify(&self, user_id: UserId, old_quota: Quota, new_quota: Quota) {
        tracing::debug!(
            "Skipped storage notification for user: {user_id}. Quota update from {old_quota:?} to {new_quota:?}"
        );
    }
}
