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
    /// Authorize access to an [`EventSharedFolder`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`EventSharedFolder`] resource (a per-event shared folder pointing
    /// into an external storage system).
    ///
    /// | Subject          | Access |
    /// | -----------------| ------ |
    /// | **Owner**        | rw     |
    /// | **Moderator**    | r-     |
    /// | **Invited-User** | r-     |
    /// | **Guest**        | --     |
    ///
    /// [`EventSharedFolder`]: opentalk_controller_api_authorization::authorization::Resource::EventSharedFolder
    pub(crate) async fn authorize_event_shared_folder(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        event_id: EventId,
    ) -> Result<Admission> {
        let acl = Acl {
            owner: Access::ReadWrite,
            moderator: Access::Read,
            invited_user: Access::Read,
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

    use super::super::test_utils::{
        EVENT_ID, USER_ID, create_authorizer_with_guest_access, create_authorizer_with_role,
    };

    #[tokio::test]
    #[rstest]
    #[case::owner_get(Owner, Get, Allowed)]
    #[case::owner_put(Owner, Put, Allowed)]
    #[case::owner_delete(Owner, Delete, Allowed)]
    #[case::moderator_get(Invited(Moderator), Get, Allowed)]
    #[case::moderator_put(Invited(Moderator), Put, Denied)]
    #[case::moderator_delete(Invited(Moderator), Delete, Denied)]
    #[case::user_get(Invited(User), Get, Allowed)]
    #[case::user_put(Invited(User), Put, Denied)]
    #[case::user_delete(Invited(User), Delete, Denied)]
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
                resource: Resource::EventSharedFolder(EVENT_ID),
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
                resource: Resource::EventSharedFolder(EVENT_ID),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
