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
    /// Authorize access to a [`Room`] resource.
    ///
    /// # Access Control List
    ///
    /// Access rights for the [`Room`] resource (`GET`, `PATCH`,
    /// `DELETE` on `/rooms/{room_id}`). Modifying or deleting a room
    /// is owner-only by design: the frontend does not surface edit
    /// controls to non-owners, so the authorizer treats any non-owner
    /// modification as denied. Moderators retain read access (the
    /// room view itself); plain invitees and invite codes are denied —
    /// they reach the room through the meeting-time `/rooms/{room_id}/start`
    /// endpoint, not through this metadata resource.
    ///
    /// | Subject      | Access |
    /// |--------------|--------|
    /// | Owner        | rw     |
    /// | Moderator    | r-     |
    /// | Invited User | --     |
    /// | Guest        | --     |
    ///
    /// [`Room`]: opentalk_controller_api_authorization::authorization::Resource::Room
    pub(crate) async fn authorize_room(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        room_id_or_alias: &RoomIdOrAlias,
    ) -> Result<Admission> {
        let acl = Acl {
            owner: Access::ReadWrite,
            moderator: Access::Read,
            invited_user: Access::None,
            guest_user: Access::None,
        };

        self.apply_acl_for_room(subjects, method, room_id_or_alias, acl)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mockall::predicate::eq;
    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Get, Patch},
        Admission::{self, Allowed, AuthenticationRequired, Denied},
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_controller_settings::test_util;
    use opentalk_inventory::{
        AuthorizationInventory,
        AuthorizationUserRole::{self, Invited, Owner, Unrelated},
        MockAuthorizationInventory, MockInventoryProvider,
    };
    use opentalk_types_common::{
        events::invites::InviteRole::{Moderator, User},
        rooms::RoomIdOrAlias,
    };
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::super::test_utils::{
        ROOM_ID, USER_ID, create_authorizer_with_guest_access, create_authorizer_with_role,
    };
    use crate::{
        OpenTalkAuthorizerBackend,
        event::test_utils::{DISABLED_FEATURES, MODULE_FEATURES},
    };

    #[tokio::test]
    #[rstest]
    #[case::owner_get(Owner, Get, Allowed)]
    #[case::owner_patch(Owner, Patch, Allowed)]
    #[case::moderator_get(Invited(Moderator), Get, Allowed)]
    #[case::moderator_patch(Invited(Moderator), Patch, Denied)]
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
                resource: Resource::Room(ROOM_ID.into()),
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
                resource: Resource::Room(ROOM_ID.into()),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(expected_admission, admission);
    }

    #[tokio::test]
    async fn multiple_subjects_first_allowed_wins() {
        // Moderator (a valid user role on the room) reads succeed; we
        // include both an invite code and the user subject to verify
        // the user branch wins without short-circuiting incorrectly.
        let mut inventory = MockAuthorizationInventory::new();
        let _ = inventory
            .expect_get_room_user_role()
            .with(
                eq(RoomIdOrAlias::from(ROOM_ID)),
                eq(USER_ID),
                eq(DISABLED_FEATURES),
                eq(MODULE_FEATURES),
            )
            .return_once(move |_, _, _, _| Ok(Invited(Moderator)));
        let inventory: Box<dyn AuthorizationInventory> = Box::new(inventory);

        let mut inventory_provider = MockInventoryProvider::new();
        let _ = inventory_provider
            .expect_get_authorization_inventory()
            .return_once(move || Ok(inventory));

        let settings_provider = test_util::settings_provider_from_example_raw_settings();
        let authorizer = OpenTalkAuthorizerBackend::new(
            Arc::new(inventory_provider),
            settings_provider,
            MODULE_FEATURES,
        );

        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([
                    Subject::from(USER_ID),
                    Subject::Unauthenticated,
                ]),
                resource: Resource::Room(ROOM_ID.into()),
                access_method: Get,
            })
            .await
            .unwrap();
        assert_eq!(Allowed, admission);
    }

    #[tokio::test]
    async fn multiple_subjects_all_denied() {
        let mut inventory = MockAuthorizationInventory::new();
        let _ = inventory
            .expect_get_room_user_role()
            .with(
                eq(RoomIdOrAlias::from(ROOM_ID)),
                eq(USER_ID),
                eq(DISABLED_FEATURES),
                eq(MODULE_FEATURES),
            )
            .return_once(move |_, _, _, _| {
                Ok(Unrelated {
                    guest_access: false,
                })
            });
        let _ = inventory
            .expect_get_room_guest_allowed()
            .with(
                eq(RoomIdOrAlias::from(ROOM_ID)),
                eq(DISABLED_FEATURES),
                eq(MODULE_FEATURES),
            )
            .return_once(move |_, _, _| Ok(false));
        let inventory: Box<dyn AuthorizationInventory> = Box::new(inventory);

        let mut inventory_provider = MockInventoryProvider::new();
        let _ = inventory_provider
            .expect_get_authorization_inventory()
            .return_once(move || Ok(inventory));

        let settings_provider = test_util::settings_provider_from_example_raw_settings();
        let authorizer = OpenTalkAuthorizerBackend::new(
            Arc::new(inventory_provider),
            settings_provider,
            MODULE_FEATURES,
        );

        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: SubjectCollection::from_iter([
                    Subject::from(USER_ID),
                    Subject::Unauthenticated,
                ]),
                resource: Resource::Room(ROOM_ID.into()),
                access_method: Get,
            })
            .await
            .unwrap();
        assert_eq!(Denied, admission);
    }
}
