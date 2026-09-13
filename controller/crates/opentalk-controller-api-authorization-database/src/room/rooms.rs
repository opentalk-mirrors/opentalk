// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_api_authorization::authorization::{Admission, SubjectCollection};

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the Rooms collection resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`Rooms`] resource.
    ///
    /// This resource allows listing and creating rooms. Any registered user can access this
    /// endpoint, regardless of the access method (read or write).
    ///
    /// | Subject  | Access |
    /// | ---------| ------ |
    /// | **User** | rw     |
    /// | Guest    | --     |
    ///
    /// [`Rooms`]: opentalk_controller_api_authorization::authorization::Resource::Rooms
    pub(crate) async fn authorize_rooms(
        &self,
        authenticated_subjects: SubjectCollection,
    ) -> Admission {
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
    #[case::user_get(Get, Allowed)]
    #[case::user_post(Post, Allowed)]
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
                resource: Resource::Rooms,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }

    #[tokio::test]
    #[rstest]
    #[case::guest_get(Get, AuthenticationRequired)]
    #[case::guest_post(Post, AuthenticationRequired)]
    async fn non_registered_guest(
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
                resource: Resource::Rooms,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
