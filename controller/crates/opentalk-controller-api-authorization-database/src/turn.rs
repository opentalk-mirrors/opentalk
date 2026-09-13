// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Authorization handlers for the deprecated TURN credentials endpoint.

use opentalk_controller_api_authorization::authorization::Admission;

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the [`Turn`] resource (`/v1/turn`).
    ///
    /// Unconditionally allowed: the endpoint is deprecated and always
    /// returns an empty response, regardless of configuration. We model
    /// it as a public resource so the authorization decision is
    /// explicit and testable rather than relying on the absence of a
    /// middleware wrap.
    ///
    /// | Subject             | Access |
    /// | ------------------- | ------ |
    /// | **Invited User**    | rw     |
    /// | **Guest**           | rw     |
    ///
    /// [`Turn`]: opentalk_controller_api_authorization::authorization::Resource::Turn
    pub(crate) const fn authorize_turn() -> Admission {
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
    async fn turn_is_unconditionally_allowed(
        #[case] subjects: SubjectCollection,
        #[case] access_method: AccessMethod,
    ) {
        let authorizer = OpenTalkAuthorizerBackend::new(
            Arc::new(MockInventoryProvider::new()),
            test_util::settings_provider_from_example_raw_settings(),
            MODULE_FEATURES,
        );

        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: subjects,
                resource: Resource::Turn,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(Allowed, admission, "Turn must always be allowed");
    }
}
