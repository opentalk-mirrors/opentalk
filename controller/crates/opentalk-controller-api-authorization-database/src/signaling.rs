// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Authorization handler for the signaling WebSocket endpoint.

use opentalk_controller_api_authorization::authorization::Admission;

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the [`Signaling`] resource
    /// (`GET` on `/v1/signaling/{token}`).
    ///
    /// Unconditionally allowed: the endpoint is gated by the signaling
    /// token in the URL path. The endpoint consumes and validates the
    /// token against the roomserver before upgrading the WebSocket, so
    /// no further check is needed at the middleware level. Modeling it
    /// here makes the "allowed by default" decision explicit and
    /// testable.
    ///
    /// | Subject          | Access |
    /// | ---------------- | ------ |
    /// | **Invited User** | rw     |
    /// | **Guest**        | rw     |
    ///
    /// [`Signaling`]: opentalk_controller_api_authorization::authorization::Resource::Signaling
    pub(crate) const fn authorize_signaling() -> Admission {
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
    use opentalk_types_common::{roomserver::Token, users::UserId};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{OpenTalkAuthorizerBackend, event::test_utils::MODULE_FEATURES};

    const USER_ID: UserId = UserId::from_u128(0x0001);
    const TOKEN: Token = Token::from_u128(0x0003);

    #[tokio::test]
    #[rstest]
    #[case::user_get(SubjectCollection::from_iter([Subject::from(USER_ID)]), Get)]
    #[case::user_post(SubjectCollection::from_iter([Subject::from(USER_ID)]), Post)]
    #[case::unauthenticated_get(SubjectCollection::from_iter([Subject::Unauthenticated]), Get)]
    #[case::unauthenticated_post(SubjectCollection::from_iter([Subject::Unauthenticated]), Post)]
    async fn signaling_is_unconditionally_allowed(
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
                resource: Resource::Signaling(TOKEN),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(Allowed, admission, "Signaling must always be allowed");
    }
}
