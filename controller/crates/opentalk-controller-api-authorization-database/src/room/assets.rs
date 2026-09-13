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
    /// Authorize access to a [`RoomAssets`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`RoomAssets`] resource (`GET`/`POST` on
    /// `/rooms/{room_id}/assets`). Moderators retain full access because they are expected to
    /// upload session materials on behalf of the owner. Invite codes are denied — see
    /// [`RoomAsset`] for the recurring-meeting rationale.
    ///
    /// | Subject      | Access |
    /// |--------------|--------|
    /// | Owner        | rw     |
    /// | Moderator    | rw     |
    /// | Invited User | r-     |
    /// | Guest        | --     |
    ///
    /// [`RoomAssets`]: opentalk_controller_api_authorization::authorization::Resource::RoomAssets
    /// [`RoomAsset`]: opentalk_controller_api_authorization::authorization::Resource::RoomAsset
    pub(crate) async fn authorize_room_assets(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        room_id_or_alias: &RoomIdOrAlias,
    ) -> Result<Admission> {
        let acl = Acl {
            owner: Access::ReadWrite,
            moderator: Access::ReadWrite,
            invited_user: Access::Read,
            guest_user: Access::None,
        };

        self.apply_acl_for_room(subjects, method, room_id_or_alias, acl)
            .await
    }
}

#[cfg(test)]
mod tests {
    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Get, Post},
        Admission::{self, Allowed, AuthenticationRequired, Denied},
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_inventory::AuthorizationUserRole::{self, Invited, Owner, Unrelated};
    use opentalk_types_common::events::invites::InviteRole::{Moderator, User};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::super::test_utils::{
        ROOM_ID, USER_ID, create_authorizer_with_guest_access, create_authorizer_with_role,
    };

    #[tokio::test]
    #[rstest]
    #[case::owner_get(Owner, Get, Allowed)]
    #[case::owner_post(Owner, Post, Allowed)]
    #[case::moderator_get(Invited(Moderator), Get, Allowed)]
    #[case::moderator_post(Invited(Moderator), Post, Allowed)]
    #[case::user_get(Invited(User), Get, Allowed)]
    #[case::user_post(Invited(User), Post, Denied)]
    #[case::unrelated_get(Unrelated { guest_access: false }, Get, Denied)]
    #[case::unrelated_get(Unrelated { guest_access: true }, Get, Denied)]
    #[case::unrelated_post(Unrelated { guest_access: false }, Post, Denied)]
    #[case::unrelated_post(Unrelated { guest_access: true }, Post, Denied)]
    async fn user(
        #[case] role: AuthorizationUserRole,
        #[case] access_method: AccessMethod,
        #[case] expected_admission: Admission,
    ) {
        let authorizer = create_authorizer_with_role(role);
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::from(USER_ID)]),
                resource: Resource::RoomAssets(ROOM_ID.into()),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }

    #[tokio::test]
    #[rstest]
    #[case::guest_access_get(true, Get, AuthenticationRequired)]
    #[case::guest_access_post(true, Post, AuthenticationRequired)]
    #[case::non_guest_access_get(false, Get, AuthenticationRequired)]
    #[case::non_guest_access_post(false, Post, AuthenticationRequired)]
    async fn unauthenticated(
        #[case] guest_access: bool,
        #[case] access_method: AccessMethod,
        #[case] expected_admission: Admission,
    ) {
        let authorizer = create_authorizer_with_guest_access(guest_access);
        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([Subject::Unauthenticated]),
                resource: Resource::RoomAssets(ROOM_ID.into()),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
