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
    /// Authorize access to a [`RoomAsset`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`RoomAsset`] resource (per-asset
    /// `GET`/`DELETE` on `/rooms/{room_id}/assets/{asset_id}`).
    ///
    /// Deletion is owner-only: a moderator who can clean up their
    /// own recordings can equally well destroy somebody else's
    /// assets, so until per-asset ownership is modelled the safer
    /// default is to keep delete with the room owner. Invite codes
    /// are denied entirely because a recurring meeting reuses the
    /// same code across sessions, and stored assets must not leak
    /// across those sessions.
    ///
    /// | Subject      | Access |
    /// |--------------|--------|
    /// | Owner        | rw     |
    /// | Moderator    | r-     |
    /// | Invited User | r-     |
    /// | Guest        | --     |
    ///
    /// [`RoomAsset`]: opentalk_controller_api_authorization::authorization::Resource::RoomAsset
    pub(crate) async fn authorize_room_asset(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        room_id_or_alias: &RoomIdOrAlias,
    ) -> Result<Admission> {
        let acl = Acl {
            owner: Access::ReadWrite,
            moderator: Access::Read,
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
        AccessMethod::{self, Delete, Get},
        Admission::{self, Allowed, AuthenticationRequired, Denied},
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_inventory::AuthorizationUserRole::{self, Invited, Owner, Unrelated};
    use opentalk_types_common::{
        assets::AssetId,
        events::invites::InviteRole::{Moderator, User},
    };
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::super::test_utils::{
        ROOM_ID, USER_ID, create_authorizer_with_guest_access, create_authorizer_with_role,
    };

    const ASSET_ID: AssetId = AssetId::from_u128(0x0004);

    #[tokio::test]
    #[rstest]
    #[case::owner_get(Owner, Get, Allowed)]
    #[case::owner_delete(Owner, Delete, Allowed)]
    #[case::moderator_get(Invited(Moderator), Get, Allowed)]
    #[case::moderator_delete(Invited(Moderator), Delete, Denied)]
    #[case::user_get(Invited(User), Get, Allowed)]
    #[case::user_delete(Invited(User), Delete, Denied)]
    #[case::unrelated_get(Unrelated { guest_access: false }, Get, Denied)]
    #[case::unrelated_get(Unrelated { guest_access: true }, Get, Denied)]
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
                resource: Resource::RoomAsset(ROOM_ID.into(), ASSET_ID),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }

    #[tokio::test]
    #[rstest]
    #[case::guest_access_get(true, Get, AuthenticationRequired)]
    #[case::guest_access_delete(true, Delete, AuthenticationRequired)]
    #[case::non_guest_access_get(false, Get, AuthenticationRequired)]
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
                resource: Resource::RoomAsset(ROOM_ID.into(), ASSET_ID),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
