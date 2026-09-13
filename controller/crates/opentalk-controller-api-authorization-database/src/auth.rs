// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Authorization handlers for the OIDC authentication endpoints.
//!
//! These endpoints sit *outside* the normal user authentication flow:
//! [`Resource::AuthLogin`] exchanges an OIDC ID token for the controller's
//! representation of the user, and [`Resource::AuthLogout`] handles
//! OIDC back-channel logout via a self-validating logout token. Both
//! must therefore be reachable without an established subject.
//!
//! Including them in the authorization middleware makes the
//! "allowed by default" decision explicit and testable rather than
//! relying on the absence of a middleware wrap.
//!
//! [`Resource::AuthLogin`]: opentalk_controller_api_authorization::authorization::Resource::AuthLogin
//! [`Resource::AuthLogout`]: opentalk_controller_api_authorization::authorization::Resource::AuthLogout

use opentalk_controller_api_authorization::authorization::Admission;

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the [`AuthLogin`] resource (`/v1/auth/login`).
    ///
    /// Unconditionally allowed: callers are unauthenticated at this point
    /// and the endpoint is the entry point of the login flow.
    ///
    /// | Subject              | Access |
    /// | -------------------- | ------ |
    /// | **Unauthenticated**  | rw     |
    /// | **User**             | rw     |
    /// | **Guest**            | rw     |
    ///
    /// [`AuthLogin`]: opentalk_controller_api_authorization::authorization::Resource::AuthLogin
    pub(crate) const fn authorize_auth_login() -> Admission {
        Admission::Allowed
    }

    /// Authorize access to the [`AuthLogout`] resource (`/v1/auth/logout`).
    ///
    /// Unconditionally allowed: OIDC back-channel logout is gated by the
    /// logout token in the request body, which the endpoint validates
    /// itself.
    ///
    /// | Subject              | Access |
    /// | -------------------- | ------ |
    /// | **Unauthenticated**  | rw     |
    /// | **User**             | rw     |
    /// | **Guest**            | rw     |
    ///
    /// [`AuthLogout`]: opentalk_controller_api_authorization::authorization::Resource::AuthLogout
    pub(crate) const fn authorize_auth_logout() -> Admission {
        Admission::Allowed
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Get, Post},
        Admission::{self, Allowed},
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_controller_settings::test_util;
    use opentalk_inventory::MockInventoryProvider;
    use opentalk_types_common::users::UserId;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{OpenTalkAuthorizerBackend, event::test_utils::MODULE_FEATURES};

    const USER_ID: UserId = UserId::from_u128(0x0001);

    fn authorizer() -> OpenTalkAuthorizerBackend {
        OpenTalkAuthorizerBackend::new(
            Arc::new(MockInventoryProvider::new()),
            test_util::settings_provider_from_example_raw_settings(),
            MODULE_FEATURES,
        )
    }

    /// Covers all subject classes (unauthenticated, user, invite code) and
    /// every common HTTP method; if access ever stops being unconditional
    /// for one of them, this test must fail.
    #[tokio::test]
    #[rstest]
    #[case::unauth_get(SubjectCollection::default(), Get)]
    #[case::unauth_post(SubjectCollection::default(), Post)]
    #[case::user_get(SubjectCollection::from_iter([Subject::from(USER_ID)]), Get)]
    #[case::user_post(SubjectCollection::from_iter([Subject::from(USER_ID)]), Post)]
    #[case::unauthenticated_get(SubjectCollection::from_iter([Subject::Unauthenticated]), Get)]
    #[case::unauthenticated_post(SubjectCollection::from_iter([Subject::Unauthenticated]), Post)]
    async fn auth_login_is_unconditionally_allowed(
        #[case] subjects: SubjectCollection,
        #[case] access_method: AccessMethod,
    ) {
        let admission = authorizer()
            .authorize(AuthorizationTarget {
                authenticated_subjects: subjects,
                resource: Resource::AuthLogin,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(Allowed, admission, "AuthLogin must always be allowed");
    }

    #[tokio::test]
    #[rstest]
    #[case::unauth_get(SubjectCollection::default(), Get)]
    #[case::unauth_post(SubjectCollection::default(), Post)]
    #[case::user_get(SubjectCollection::from_iter([Subject::from(USER_ID)]), Get)]
    #[case::user_post(SubjectCollection::from_iter([Subject::from(USER_ID)]), Post)]
    #[case::unauthenticated_get(SubjectCollection::from_iter([Subject::Unauthenticated]), Get)]
    #[case::unauthenticated_post(SubjectCollection::from_iter([Subject::Unauthenticated]), Post)]
    async fn auth_logout_is_unconditionally_allowed(
        #[case] subjects: SubjectCollection,
        #[case] access_method: AccessMethod,
    ) {
        let admission: Admission = authorizer()
            .authorize(AuthorizationTarget {
                authenticated_subjects: subjects,
                resource: Resource::AuthLogout,
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(Allowed, admission, "AuthLogout must always be allowed");
    }
}
