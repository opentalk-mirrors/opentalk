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
    /// Authorize access to an [`EventInstance`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`EventInstance`] resource (a single instance of a recurring event,
    /// identified by `instance_id`). Same access model as [`Event`] — the `instance_id` segment has
    /// no impact on authorization, only on which instance the request mutates.
    ///
    /// API methods exposed are `GET` and `PATCH`. Only the owner may `PATCH`; any associated
    /// user subject (Owner, Invited(_)) may `GET`. Invite codes are denied — they reach the room
    /// through the meeting-time `/rooms/{room_id}/start` endpoint, not through this resource.
    ///
    /// | Subject                 | Access |
    /// | ----------------------- | ------ |
    /// | **Owner**               | rw     |
    /// | **Moderator**           | r-     |
    /// | **Invited-User**        | r-     |
    /// | **Guest**               | --     |
    ///
    /// [`EventInstance`]: opentalk_controller_api_authorization::authorization::Resource::EventInstance
    /// [`Event`]: opentalk_controller_api_authorization::authorization::Resource::Event
    pub(crate) async fn authorize_event_instance(
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
        AccessMethod::{self, Get, Patch},
        Admission::{self, Allowed, AuthenticationRequired, Denied},
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_inventory::AuthorizationUserRole::{self, Invited, Owner, Unrelated};
    use opentalk_types_api_v1::events::InstanceId;
    use opentalk_types_common::{
        events::invites::InviteRole::{Moderator, User},
        time::Timestamp,
    };
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::super::test_utils::{
        EVENT_ID, USER_ID, create_authorizer_with_guest_access, create_authorizer_with_role,
    };

    /// A constant `InstanceId` used by tests. Its value is irrelevant
    /// to authorization — every event instance inherits the event's
    /// ACL — but `InstanceId`'s fields are private, so we construct
    /// it from a stable timestamp.
    fn instance_id() -> InstanceId {
        InstanceId::from(Timestamp::unix_epoch())
    }

    #[tokio::test]
    #[rstest]
    #[case::owner_get(Owner, Get, Allowed)]
    #[case::owner_patch(Owner, Patch, Allowed)]
    #[case::moderator_get(Invited(Moderator), Get, Allowed)]
    #[case::moderator_patch(Invited(Moderator), Patch, Denied)]
    #[case::user_get(Invited(User), Get, Allowed)]
    #[case::user_patch(Invited(User), Patch, Denied)]
    #[case::unrelated_get(Unrelated { guest_access: false }, Get, Denied)]
    #[case::unrelated_get(Unrelated { guest_access: true }, Get, Denied)]
    #[case::unrelated_patch(Unrelated { guest_access: false }, Patch, Denied)]
    #[case::unrelated_patch(Unrelated { guest_access: true }, Patch, Denied)]
    async fn user(
        #[case] role: AuthorizationUserRole,
        #[case] access_method: AccessMethod,
        #[case] expected_admission: Admission,
    ) {
        let authorizer = create_authorizer_with_role(role);
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::from(USER_ID)]),
                resource: Resource::EventInstance(EVENT_ID, instance_id()),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }

    #[tokio::test]
    #[rstest]
    #[case::guest_access_get(true, Get, AuthenticationRequired)]
    #[case::guest_access_patch(true, Patch, AuthenticationRequired)]
    #[case::non_guest_access_get(false, Get, AuthenticationRequired)]
    #[case::non_guest_access_patch(false, Patch, AuthenticationRequired)]
    async fn unauthenticated(
        #[case] guest_access: bool,
        #[case] access_method: AccessMethod,
        #[case] expected_admission: Admission,
    ) {
        let authorizer = create_authorizer_with_guest_access(guest_access);
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::Unauthenticated]),
                resource: Resource::EventInstance(EVENT_ID, instance_id()),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
