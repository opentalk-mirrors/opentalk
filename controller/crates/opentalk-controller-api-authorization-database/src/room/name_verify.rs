// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_controller_api_authorization::authorization::{Admission, SubjectCollection};

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the [`RoomNameVerify`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`RoomNameVerify`] resource.
    ///
    /// Any registered user can access this endpoint, regardless of the access method (read or write).
    ///
    /// This resource allows checking if a room name is available. Knowledge of a room name allows anybody to join the
    /// room when room alias suffix is disabled and the room has guest access enabled and no password. Only registered
    /// users need to check room name availability because unregistered users can not create rooms.
    ///
    /// | Subject                 | Access |
    /// | ----------------------- | ------ |
    /// | **User**                | rw     |
    /// | **Unauthenticated**     | --     |
    ///
    /// [`RoomNameVerify`]: opentalk_controller_api_authorization::authorization::Resource::RoomNameVerify
    pub(crate) fn authorize_room_name_verify(subjects: SubjectCollection) -> Admission {
        Self::require_read_write_user(subjects)
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
    #[case::user_get(SubjectCollection::from_iter([Subject::from(USER_ID)]), Get, Allowed)]
    #[case::user_post(SubjectCollection::from_iter([Subject::from(USER_ID)]), Post, Allowed)]
    #[case::unauthenticated_get(SubjectCollection::from_iter([Subject::Unauthenticated]), Get, AuthenticationRequired)]
    #[case::unauthenticated_post(SubjectCollection::from_iter([Subject::Unauthenticated]), Post, AuthenticationRequired)]
    async fn authorize_room_name_verify(
        #[case] subjects: SubjectCollection,
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
                authenticated_subjects: subjects,
                resource: Resource::RoomNameVerify,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
