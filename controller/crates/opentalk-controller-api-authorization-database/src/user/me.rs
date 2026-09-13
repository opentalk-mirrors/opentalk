// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_api_authorization::authorization::{Admission, SubjectCollection};

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the [`UserMe`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`UserMe`] resource. This resource exposes
    /// the current user's information, so any registered user can access
    /// it; invite codes can not.
    ///
    /// | Subject   | Access |
    /// | ----------| ------ |
    /// | **User**  | rw     |
    /// | **Guest** | --     |
    ///
    /// [`UserMe`]: opentalk_controller_api_authorization::authorization::Resource::UserMe
    pub(crate) fn authorize_user_me(&self, authenticated_subjects: SubjectCollection) -> Admission {
        Self::require_read_write_user(authenticated_subjects)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Get, Post},
        Admission::{self, Allowed, AuthenticationRequired},
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
    #[case::post(Post, Allowed)]
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
                resource: Resource::UserMe,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }

    #[tokio::test]
    #[rstest]
    #[case::get(Get, AuthenticationRequired)]
    #[case::post(Post, AuthenticationRequired)]
    async fn unauthenticated(
        #[case] access_method: AccessMethod,
        #[case] expected_admission: Admission,
    ) {
        let inventory_provider = MockInventoryProvider::new();
        let settings_provider = test_util::settings_provider_from_example_raw_settings();
        let authorizer = OpenTalkAuthorizerBackend::new(
            Arc::new(inventory_provider),
            settings_provider,
            MODULE_FEATURES,
        );
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::Unauthenticated]),
                resource: Resource::UserMe,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
