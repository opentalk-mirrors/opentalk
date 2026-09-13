// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_api_authorization::authorization::{
    AccessMethod, Admission, SubjectCollection,
};
use opentalk_types_common::rooms::RoomIdOrAlias;

use crate::{
    OpenTalkAuthorizerBackend, Result,
    common::acl::{Access, Acl},
};

impl OpenTalkAuthorizerBackend {
    /// Authorize access to a [`RoomStreamingTarget`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`RoomStreamingTarget`] resource
    /// (`GET`/`PATCH`/`DELETE` on
    /// `/rooms/{room_id}/streaming_targets/{streaming_target_id}`).
    /// The streaming target contains the upstream streaming key —
    /// effectively a credential — so the matrix is owner-only: no
    /// moderator, invitee, or invite-code subject may read or
    /// modify it.
    ///
    /// | Subject               | Access |
    /// |-----------------------|--------|
    /// | Owner                 | rw     |
    /// | Moderator             | --     |
    /// | User                  | --     |
    /// | Guest                 | --     |
    ///
    /// [`RoomStreamingTarget`]: opentalk_controller_api_authorization::authorization::Resource::RoomStreamingTarget
    pub(crate) async fn authorize_room_streaming_target(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        room_id_or_alias: &RoomIdOrAlias,
    ) -> Result<Admission> {
        let acl = Acl {
            owner: Access::ReadWrite,
            moderator: Access::None,
            invited_user: Access::None,
            guest_user: Access::None,
        };

        self.apply_acl_for_room(subjects, method, room_id_or_alias, acl)
            .await
    }
}

#[cfg(test)]
mod tests {
    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Delete, Get, Patch},
        Admission::{self, Allowed, AuthenticationRequired, Denied},
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_inventory::AuthorizationUserRole::{self, Invited, Owner, Unrelated};
    use opentalk_types_common::{
        events::invites::InviteRole::{Moderator, User},
        streaming::StreamingTargetId,
    };
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::super::test_utils::{
        ROOM_ID, USER_ID, create_authorizer_with_guest_access, create_authorizer_with_role,
    };

    const STREAMING_TARGET_ID: StreamingTargetId = StreamingTargetId::from_u128(0x0005);

    #[tokio::test]
    #[rstest]
    #[case::owner_get(Owner, Get, Allowed)]
    #[case::owner_patch(Owner, Patch, Allowed)]
    #[case::owner_delete(Owner, Delete, Allowed)]
    #[case::moderator_get(Invited(Moderator), Get, Denied)]
    #[case::moderator_patch(Invited(Moderator), Patch, Denied)]
    #[case::moderator_delete(Invited(Moderator), Delete, Denied)]
    #[case::user_get(Invited(User), Get, Denied)]
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
                resource: Resource::RoomStreamingTarget(ROOM_ID.into(), STREAMING_TARGET_ID),
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
                resource: Resource::RoomStreamingTarget(ROOM_ID.into(), STREAMING_TARGET_ID),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
