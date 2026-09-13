// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Authorization handler for the invite verification endpoint.

use opentalk_controller_api_authorization::authorization::Admission;

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the [`InviteVerify`] resource
    /// (`POST` on `/v1/invite/verify`).
    ///
    /// Unconditionally allowed: the endpoint validates the invite code
    /// carried in the JSON body itself, so no further check is needed
    /// at the middleware level. Modeling it here makes the "allowed by
    /// default" decision explicit and testable.
    ///
    /// | Subject             | Access |
    /// | ------------------- | ------ |
    /// | **User**            | rw     |
    /// | **Guest**           | rw     |
    ///
    /// [`InviteVerify`]: opentalk_controller_api_authorization::authorization::Resource::InviteVerify
    pub(crate) const fn authorize_invite_verify() -> Admission {
        Admission::Allowed
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Get, Post},
        Admission::Allowed,
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
    #[case::user_get(SubjectCollection::from_iter([Subject::from(USER_ID)]), Get)]
    #[case::user_post(SubjectCollection::from_iter([Subject::from(USER_ID)]), Post)]
    #[case::unauthenticated_get(SubjectCollection::from_iter([Subject::Unauthenticated]), Get)]
    #[case::unauthenticated_post(SubjectCollection::from_iter([Subject::Unauthenticated]), Post)]
    async fn invite_verify_is_unconditionally_allowed(
        #[case] subjects: SubjectCollection,
        #[case] access_method: AccessMethod,
    ) {
        // The authorizer must not consult the inventory.
        let authorizer = OpenTalkAuthorizerBackend::new(
            Arc::new(MockInventoryProvider::new()),
            test_util::settings_provider_from_example_raw_settings(),
            MODULE_FEATURES,
        );

        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: subjects,
                resource: Resource::InviteVerify,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(Allowed, admission, "InviteVerify must always be allowed");
    }
}
