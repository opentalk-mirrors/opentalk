// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_api_authorization::authorization::{
    AccessMethod, Admission, SubjectCollection,
};

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the [`UserMePendingInvites`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`UserMePendingInvites`] resource (`GET`
    /// on `/users/me/pending_invites`). The endpoint lists the
    /// caller's pending event invites. It is read-only — invite
    /// acceptance/decline happens via the per-event `EventInvite`
    /// resource, not here — and invite codes have no access.
    ///
    /// | Subject    | Access |
    /// | ---------- | ------ |
    /// | **User**   | r-     |
    /// | **Guest**  | --     |
    ///
    /// [`UserMePendingInvites`]: opentalk_controller_api_authorization::authorization::Resource::UserMePendingInvites
    pub(crate) fn authorize_user_me_pending_invites(
        &self,
        authenticated_subjects: SubjectCollection,
        access_method: AccessMethod,
    ) -> Admission {
        Self::require_read_user(authenticated_subjects, access_method)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Get, Post},
        Admission::{self, Allowed, AuthenticationRequired, Denied},
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_controller_settings::test_util;
    use opentalk_inventory::MockInventoryProvider;
    use opentalk_types_common::users::UserId;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{OpenTalkAuthorizerBackend, event::test_utils::MODULE_FEATURES};

    const USER_ID: UserId = UserId::from_u128(0x0001);

    #[tokio::test]
    #[rstest]
    #[case::get(Get, Allowed)]
    #[case::post(Post, Denied)]
    async fn user(#[case] access_method: AccessMethod, #[case] expected_admission: Admission) {
        let inventory_provider = MockInventoryProvider::new();
        let authorizer = OpenTalkAuthorizerBackend::new(
            Arc::new(inventory_provider),
            test_util::settings_provider_from_example_raw_settings(),
            MODULE_FEATURES,
        );
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::from(USER_ID)]),
                resource: Resource::UserMePendingInvites,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }

    #[tokio::test]
    #[rstest]
    #[case::get(Get, AuthenticationRequired)]
    #[case::post(Post, Denied)]
    async fn unauthenticated(
        #[case] access_method: AccessMethod,
        #[case] expected_admission: Admission,
    ) {
        let inventory_provider = MockInventoryProvider::new();
        let authorizer = OpenTalkAuthorizerBackend::new(
            Arc::new(inventory_provider),
            test_util::settings_provider_from_example_raw_settings(),
            MODULE_FEATURES,
        );
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::Unauthenticated]),
                resource: Resource::UserMePendingInvites,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
