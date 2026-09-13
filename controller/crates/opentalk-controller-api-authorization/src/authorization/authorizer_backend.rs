// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use async_trait::async_trait;

use super::{
    Admission, AuthorizationChange, AuthorizationChangeError, AuthorizationError,
    AuthorizationTarget,
};

/// A trait for implementing authorization queries against the OpenTalk Controller API.
#[cfg_attr(feature = "mockall", mockall::automock)]
#[async_trait]
pub trait AuthorizerBackend: Send + Sync {
    /// Attempt to authorize for a specific resource as a subject.
    async fn authorize(&self, target: AuthorizationTarget)
    -> Result<Admission, AuthorizationError>;

    /// Apply a changeset to the authorization backend data
    ///
    /// This takes `&self` so the [`Authorizer`](super::Authorizer) can hold the
    /// backend behind a plain `Arc` instead of a lock. If a future backend
    /// implementation ever needs interior mutability to update a cache on
    /// change, it must provide its own synchronization (e.g. `Mutex`,
    /// `RwLock`, ...), and this signature may need to be reconsidered.
    async fn apply_changes(
        &self,
        changeset: &[AuthorizationChange],
    ) -> Result<(), AuthorizationChangeError>;
}
