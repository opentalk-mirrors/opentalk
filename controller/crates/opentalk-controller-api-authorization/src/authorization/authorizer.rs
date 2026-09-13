// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::Arc;

use super::{
    Admission, AuthorizationChange, AuthorizationChangeError, AuthorizationError,
    AuthorizationTarget, AuthorizerBackend,
};

/// A handle holding an authorizer. Contains a thread-safe reference to a
/// `dyn` [`AuthorizerBackend`] implementation.
///
/// Both [`AuthorizerBackend::authorize`] and [`AuthorizerBackend::apply_changes`]
/// take `&self`, so no locking is required here. If a future backend ever needs
/// interior mutability (for example to invalidate a cache in `apply_changes`),
/// it must provide its own synchronization.
#[derive(Clone)]
pub struct Authorizer {
    backend: Arc<dyn AuthorizerBackend>,
}

impl std::fmt::Debug for Authorizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Authorizer")
    }
}

impl Authorizer {
    /// Create a new authorizer
    pub fn new<B: AuthorizerBackend + 'static>(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
        }
    }

    /// Request authorization for a specific [`AuthorizationTarget`].
    pub async fn authorize(
        &self,
        target: AuthorizationTarget,
    ) -> Result<Admission, AuthorizationError> {
        self.backend.authorize(target).await
    }

    /// Apply a changeset to the authorization backend data
    pub async fn apply_changes(
        &self,
        changeset: &[AuthorizationChange],
    ) -> Result<(), AuthorizationChangeError> {
        self.backend.apply_changes(changeset).await
    }

    /// Apply a change to the authorization backend data
    pub async fn apply_change(
        &self,
        change: &AuthorizationChange,
    ) -> Result<(), AuthorizationChangeError> {
        self.apply_changes(std::slice::from_ref(change)).await
    }
}
