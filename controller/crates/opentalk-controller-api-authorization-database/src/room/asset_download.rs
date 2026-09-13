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
    /// Authorize access to a [`RoomAssetDownload`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`RoomAssetDownload`] resource (`GET`
    /// on `/rooms/{room_id}/assets/{asset_id}/download`). The
    /// matrix is identical to [`RoomAsset`] but expressed as
    /// read-only on every row — the endpoint serves a signed
    /// download URL, never modifies anything, and invite codes are
    /// denied for the same recurring-meeting reasons.
    ///
    /// | Subject      | Access |
    /// |--------------|--------|
    /// | Owner        | r-     |
    /// | Moderator    | r-     |
    /// | Invited User | r-     |
    /// | Guest        | --     |
    ///
    /// [`RoomAssetDownload`]: opentalk_controller_api_authorization::authorization::Resource::RoomAssetDownload
    /// [`RoomAsset`]: opentalk_controller_api_authorization::authorization::Resource::RoomAsset
    pub(crate) async fn authorize_room_asset_download(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        room_id_or_alias: RoomIdOrAlias,
    ) -> Result<Admission> {
        let acl = Acl {
            owner: Access::Read,
            moderator: Access::Read,
            invited_user: Access::Read,
            guest_user: Access::None,
        };

        self.apply_acl_for_room(subjects, method, &room_id_or_alias, acl)
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
    #[case::owner_post(Owner, Post, Denied)]
    #[case::moderator_get(Invited(Moderator), Get, Allowed)]
    #[case::moderator_post(Invited(Moderator), Post, Denied)]
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
                resource: Resource::RoomAssetDownload(ROOM_ID.into(), ASSET_ID),
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
                resource: Resource::RoomAssetDownload(ROOM_ID.into(), ASSET_ID),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }
}
