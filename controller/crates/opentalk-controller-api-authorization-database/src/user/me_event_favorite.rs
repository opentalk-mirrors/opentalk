// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_api_authorization::authorization::{
    AccessMethod, Admission, SubjectCollection,
};
use opentalk_types_common::events::EventId;

use crate::{
    OpenTalkAuthorizerBackend, Result,
    common::acl::{Access, Acl},
};

impl OpenTalkAuthorizerBackend {
    /// Authorize access to a [`UserMeEventFavorite`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`UserMeEventFavorite`] resource (the per-user favourite-event toggle,
    /// expressed against an `event_id`). Any event member — owner or any invited user — can
    /// favourite or unfavourite an event. Invite codes are denied.
    ///
    /// | Subject           | Access |
    /// | ------------------| ------ |
    /// | **Owner**         | rw     |
    /// | **Moderator**     | rw     |
    /// | **Invited-User**  | rw     |
    /// | **Guest**         | --     |
    ///
    /// Although a "favourite" is conceptually per-user state, the authorization decision is keyed
    /// on event membership and so dispatches to [`require_event_member`], the shared event-resource
    /// helper. Lives under `user/` because it is the user's favorite, not the event's; calls into
    /// `event/` cross- module via `pub(crate)`.
    ///
    /// [`UserMeEventFavorite`]: opentalk_controller_api_authorization::authorization::Resource::UserMeEventFavorite
    /// [`require_event_member`]: Self::require_event_member
    pub(crate) async fn authorize_user_me_event_favorite(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        event_id: EventId,
    ) -> Result<Admission> {
        let acl = Acl {
            owner: Access::ReadWrite,
            moderator: Access::ReadWrite,
            invited_user: Access::ReadWrite,
            guest_user: Access::None,
        };

        self.apply_acl_for_event(subjects, method, event_id, acl)
            .await
    }
}

#[cfg(test)]
mod tests {
    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Delete, Get, Put},
        Admission::{self, Allowed, AuthenticationRequired, Denied},
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_inventory::AuthorizationUserRole::{self, Invited, Owner, Unrelated};
    use opentalk_types_common::events::invites::InviteRole::{Moderator, User};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::event::test_utils::{
        EVENT_ID, USER_ID, create_authorizer_with_guest_access, create_authorizer_with_role,
    };

    #[tokio::test]
    #[rstest]
    #[case::owner_get(Owner, Get, Allowed)]
    #[case::owner_put(Owner, Put, Allowed)]
    #[case::owner_delete(Owner, Delete, Allowed)]
    #[case::moderator_get(Invited(Moderator), Get, Allowed)]
    #[case::moderator_put(Invited(Moderator), Put, Allowed)]
    #[case::moderator_delete(Invited(Moderator), Delete, Allowed)]
    #[case::invited_get(Invited(User), Get, Allowed)]
    #[case::invited_put(Invited(User), Put, Allowed)]
    #[case::invited_delete(Invited(User), Delete, Allowed)]
    #[case::unrelated_get(Unrelated { guest_access: false }, Get, Denied)]
    #[case::unrelated_get(Unrelated { guest_access: true }, Get, Denied)]
    #[case::unrelated_put(Unrelated { guest_access: false }, Put, Denied)]
    #[case::unrelated_put(Unrelated { guest_access: true }, Put, Denied)]
    #[case::unrelated_delete(Unrelated { guest_access: false }, Delete, Denied)]
    #[case::unrelated_delete(Unrelated { guest_access: true }, Delete, Denied)]
    async fn user(
        #[case] role: AuthorizationUserRole,
        #[case] access_method: AccessMethod,
        #[case] expected_admission: Admission,
    ) {
        let authorizer = create_authorizer_with_role(role);
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::from(USER_ID)]),
                resource: Resource::UserMeEventFavorite(EVENT_ID),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }

    #[tokio::test]
    #[rstest]
    #[case::guest_access_get(true, Get, AuthenticationRequired)]
    #[case::guest_access_put(true, Put, AuthenticationRequired)]
    #[case::guest_access_delete(true, Delete, AuthenticationRequired)]
    #[case::non_guest_access_get(false, Get, AuthenticationRequired)]
    #[case::non_guest_access_put(false, Put, AuthenticationRequired)]
    #[case::non_guest_access_delete(false, Delete, AuthenticationRequired)]
    async fn unauthenticated(
        #[case] guest_access: bool,
        #[case] access_method: AccessMethod,
        #[case] expected_admission: Admission,
    ) {
        let authorizer = create_authorizer_with_guest_access(guest_access);
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::Unauthenticated]),
                resource: Resource::UserMeEventFavorite(EVENT_ID),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
