// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Cross-resource ACL helpers shared by `event/`, `room/` and `user/`.

use acl::Acl;
use opentalk_controller_api_authorization::authorization::{
    AccessMethod, Admission, Subject, SubjectCollection,
};
use opentalk_inventory::AuthorizationUserRole as Role;
use opentalk_types_common::{
    events::{EventId, invites::InviteRole},
    rooms::RoomIdOrAlias,
};

use crate::{OpenTalkAuthorizerBackend, Result};

impl OpenTalkAuthorizerBackend {
    /// Helper for ACL pattern: any registered user can access; invite codes can not.
    ///
    /// This helper is used by resources that don't fit the Owner/Moderator/Member/Invited model,
    /// but should be accessible to any registered user, regardless of access method. Examples
    /// include the `Rooms` and `Events` listing endpoints, or the `UserMe` and `UserProfile`
    /// resources that expose write paths.
    pub(crate) fn require_read_write_user(subjects: SubjectCollection) -> Admission {
        if subjects.contains_any_user() {
            Admission::Allowed
        } else {
            Admission::AuthenticationRequired
        }
    }

    /// Helper for ACL pattern: any registered user can read; invite codes can not; nobody writes.
    ///
    /// Read-only variant of [`Self::require_write_user`] for endpoints that don't fit the
    /// Owner/Moderator/Invited/InviteCode model and expose no write methods. Examples include
    /// `UserFind` (`GET /v1/users/find`) and the `UserMeTariff` / `UserMePendingInvites` /
    /// `UserMeAssets` endpoints, which all only expose `GET`.
    pub(crate) fn require_read_user(
        subjects: SubjectCollection,
        method: AccessMethod,
    ) -> Admission {
        match (subjects.contains_any_user(), method.is_read_only()) {
            (true, true) => Admission::Allowed,
            (false, true) => Admission::AuthenticationRequired,
            _ => Admission::Denied,
        }
    }

    pub(crate) async fn apply_acl_for_event(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        event_id: EventId,
        acl: Acl,
    ) -> Result<Admission> {
        let mut inventory = self.inventory.get_authorization_inventory().await?;

        let module_features = self.module_features.clone();
        let disabled_features = self.settings.get().defaults.disabled_features.clone();

        for user_id in subjects.all_user_ids() {
            let role = inventory
                .get_event_user_role(
                    event_id,
                    user_id,
                    disabled_features.clone(),
                    module_features.clone(),
                )
                .await?;
            let admission = acl.apply(role.into(), method);
            if admission.is_allowed() {
                return Ok(admission);
            }
        }

        if subjects.contains(&Subject::Unauthenticated) {
            let guest_access = inventory
                .get_event_guest_allowed(
                    event_id,
                    disabled_features.clone(),
                    module_features.clone(),
                )
                .await?;
            let admission = acl.apply(acl::Subject::Unregistered { guest_access }, method);
            if admission.is_allowed() {
                return Ok(admission);
            }
        }
        if !subjects.contains_any_user() {
            Ok(Admission::AuthenticationRequired)
        } else {
            Ok(Admission::Denied)
        }
    }

    pub(crate) async fn apply_acl_for_room(
        &self,
        subjects: SubjectCollection,
        method: AccessMethod,
        room_id_or_alias: &RoomIdOrAlias,
        acl: Acl,
    ) -> Result<Admission> {
        let mut inventory = self.inventory.get_authorization_inventory().await?;

        let module_features = self.module_features.clone();
        let disabled_features = self.settings.get().defaults.disabled_features.clone();

        for user_id in subjects.all_user_ids() {
            let role = inventory
                .get_room_user_role(
                    room_id_or_alias,
                    user_id,
                    disabled_features.clone(),
                    module_features.clone(),
                )
                .await?;
            let admission = acl.apply(role.into(), method);
            if admission.is_allowed() {
                return Ok(admission);
            }
        }

        if subjects.contains(&Subject::Unauthenticated) {
            let guest_access = inventory
                .get_room_guest_allowed(
                    room_id_or_alias,
                    disabled_features.clone(),
                    module_features.clone(),
                )
                .await?;
            let admission = acl.apply(acl::Subject::Unregistered { guest_access }, method);
            if admission.is_allowed() {
                return Ok(admission);
            }
        }
        if !subjects.contains_any_user() {
            Ok(Admission::AuthenticationRequired)
        } else {
            Ok(Admission::Denied)
        }
    }
}

pub(crate) mod acl {
    use super::*;

    #[derive(Debug, Clone)]
    pub enum Access {
        ReadWrite,
        Read,
        None,
    }

    /// # Access Control List
    ///
    /// Used to enforce [`Access`] to specific resources, i.e., API endpoints that relate to created
    /// events or rooms. Note that a [`Subject`] that does not map to [`Acl::owner`],
    /// [`Acl::moderator`], [`Acl::invited_user`], or [`Acl::guest_user`] is denied
    /// admission ([`Admission::Denied`]) by default.
    #[derive(Debug, Clone)]
    pub struct Acl {
        pub owner: Access,
        pub moderator: Access,
        pub invited_user: Access,
        pub guest_user: Access,
    }

    impl Acl {
        pub(super) fn apply(&self, subject: Subject, method: AccessMethod) -> Admission {
            let access = self.get_access(subject);

            match (access, method) {
                (Access::ReadWrite, _) => Admission::Allowed,
                (Access::Read, method) if method.is_read_only() => Admission::Allowed,
                (Access::Read, _) | (Access::None, _) => Admission::Denied,
            }
        }

        fn get_access(&self, subject: Subject) -> &Access {
            match subject {
                Subject::User(role) => match role {
                    Role::Owner => &self.owner,
                    Role::Invited(InviteRole::Moderator) => &self.moderator,
                    Role::Invited(InviteRole::User) => &self.invited_user,
                    Role::Unrelated { guest_access: true } => &self.guest_user,
                    Role::Unrelated {
                        guest_access: false,
                    } => &Access::None,
                },
                Subject::Unregistered { guest_access: true } => &self.guest_user,
                Subject::Unregistered {
                    guest_access: false,
                } => &Access::None,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub(super) enum Subject {
        User(Role),
        Unregistered { guest_access: bool },
    }

    impl From<Role> for Subject {
        fn from(value: Role) -> Self {
            Self::User(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use AccessMethod::{Get, Post};
    use Admission::{Allowed, Denied};
    use acl::Subject::User;
    use opentalk_controller_api_authorization::authorization::Admission::AuthenticationRequired;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;
    use crate::{common::acl::Access, event, room};

    #[rstest]
    #[case::apply_owner(User(Role::Owner), Allowed, Allowed)]
    #[case::apply_moderator(User(Role::Invited(InviteRole::Moderator)), Denied, Denied)]
    #[case::apply_invited_user(User(Role::Invited(InviteRole::User)), Denied, Denied)]
    #[case::apply_unrelated_user_no_guest(User(Role::Unrelated{guest_access:false}), Denied, Denied)]
    #[case::apply_unrelated_user_guest(User(Role::Unrelated{guest_access:true}), Denied, Denied)]
    fn apply_acl_read_write_owner(
        #[case] sub: acl::Subject,
        #[case] expected_get: Admission,
        #[case] expected_post: Admission,
    ) {
        let acl = Acl {
            owner: Access::ReadWrite,
            moderator: Access::None,
            invited_user: Access::None,
            guest_user: Access::None,
        };

        assert_eq!(expected_get, acl.apply(sub.clone(), Get));
        assert_eq!(expected_post, acl.apply(sub, Post));
    }

    #[rstest]
    #[case::apply_owner(User(Role::Owner), Denied, Denied)]
    #[case::apply_moderator(User(Role::Invited(InviteRole::Moderator)), Allowed, Allowed)]
    #[case::apply_invited_user(User(Role::Invited(InviteRole::User)), Denied, Denied)]
    #[case::apply_unrelated_user_no_guest(User(Role::Unrelated{guest_access:false}), Denied, Denied)]
    #[case::apply_unrelated_user_guest(User(Role::Unrelated{guest_access:true}), Denied, Denied)]
    fn apply_acl_read_write_moderator(
        #[case] sub: acl::Subject,
        #[case] expected_get: Admission,
        #[case] expected_post: Admission,
    ) {
        let acl = Acl {
            owner: Access::None,
            moderator: Access::ReadWrite,
            invited_user: Access::None,
            guest_user: Access::None,
        };

        assert_eq!(expected_get, acl.apply(sub.clone(), Get));
        assert_eq!(expected_post, acl.apply(sub, Post));
    }

    #[rstest]
    #[case::apply_owner(User(Role::Owner), Denied, Denied)]
    #[case::apply_moderator(User(Role::Invited(InviteRole::Moderator)), Denied, Denied)]
    #[case::apply_invited_user(User(Role::Invited(InviteRole::User)), Allowed, Allowed)]
    #[case::apply_unrelated_user_no_guest(User(Role::Unrelated{guest_access:false}), Denied, Denied)]
    #[case::apply_unrelated_user_guest(User(Role::Unrelated{guest_access:true}), Denied, Denied)]
    fn apply_acl_read_write_invited_user(
        #[case] sub: acl::Subject,
        #[case] expected_get: Admission,
        #[case] expected_post: Admission,
    ) {
        let acl = Acl {
            owner: Access::None,
            moderator: Access::None,
            invited_user: Access::ReadWrite,
            guest_user: Access::None,
        };

        assert_eq!(expected_get, acl.apply(sub.clone(), Get));
        assert_eq!(expected_post, acl.apply(sub, Post));
    }

    #[rstest]
    #[case::apply_owner(User(Role::Owner), Denied, Denied)]
    #[case::apply_moderator(User(Role::Invited(InviteRole::Moderator)), Denied, Denied)]
    #[case::apply_invited_user(User(Role::Invited(InviteRole::User)), Denied, Denied)]
    #[case::apply_unrelated_user_no_guest(User(Role::Unrelated{guest_access:false}), Denied, Denied)]
    #[case::apply_unrelated_user_guest(User(Role::Unrelated{guest_access:true}), Allowed, Allowed)]
    fn apply_acl_read_write_unregistered(
        #[case] sub: acl::Subject,
        #[case] expected_get: Admission,
        #[case] expected_post: Admission,
    ) {
        let acl = Acl {
            owner: Access::None,
            moderator: Access::None,
            invited_user: Access::None,
            guest_user: Access::ReadWrite,
        };

        assert_eq!(expected_get, acl.apply(sub.clone(), Get));
        assert_eq!(expected_post, acl.apply(sub, Post));
    }

    #[rstest]
    #[case::apply_owner(User(Role::Owner), Denied, Denied)]
    #[case::apply_moderator(User(Role::Invited(InviteRole::Moderator)), Denied, Denied)]
    #[case::apply_invited_user(User(Role::Invited(InviteRole::User)), Denied, Denied)]
    #[case::apply_unrelated_user_no_guest(User(Role::Unrelated{guest_access:false}), Denied, Denied)]
    #[case::apply_unrelated_user_guest(User(Role::Unrelated{guest_access:true}), Denied, Denied)]
    fn apply_acl_read_write_none(
        #[case] sub: acl::Subject,
        #[case] expected_get: Admission,
        #[case] expected_post: Admission,
    ) {
        let acl = Acl {
            owner: Access::None,
            moderator: Access::None,
            invited_user: Access::None,
            guest_user: Access::None,
        };

        assert_eq!(expected_get, acl.apply(sub.clone(), Get));
        assert_eq!(expected_post, acl.apply(sub, Post));
    }

    #[rstest]
    #[case::user_get(Subject::User(event::test_utils::USER_ID), Get, Allowed)]
    #[case::user_post(Subject::User(event::test_utils::USER_ID), Post, Denied)]
    #[case::unauthenticated_get(Subject::Unauthenticated, Get, AuthenticationRequired)]
    #[case::unauthenticated_post(Subject::Unauthenticated, Post, Denied)]
    fn require_read_user(
        #[case] sub: Subject,
        #[case] method: AccessMethod,
        #[case] admission: Admission,
    ) {
        let subjects = SubjectCollection::from_iter([sub]);
        assert_eq!(
            OpenTalkAuthorizerBackend::require_read_user(subjects, method),
            admission
        );
    }

    #[rstest]
    #[case::user_get(Subject::User(event::test_utils::USER_ID), Allowed)]
    #[case::unauthenticated_get(Subject::Unauthenticated, AuthenticationRequired)]
    fn require_write_user(#[case] sub: Subject, #[case] admission: Admission) {
        let subjects = SubjectCollection::from_iter([sub]);
        assert_eq!(
            OpenTalkAuthorizerBackend::require_read_write_user(subjects),
            admission
        );
    }

    #[test]
    fn require_authentication_for_require_read_user() {
        let empty_subjects = SubjectCollection::from_iter([]);
        assert_eq!(
            OpenTalkAuthorizerBackend::require_read_user(empty_subjects, Get),
            Admission::AuthenticationRequired
        )
    }

    #[test]
    fn require_authentication_for_require_read_write_user() {
        let empty_subjects = SubjectCollection::from_iter([]);
        assert_eq!(
            OpenTalkAuthorizerBackend::require_read_write_user(empty_subjects),
            Admission::AuthenticationRequired
        )
    }

    #[tokio::test]
    async fn require_authentication_to_apply_acl_for_event() {
        let empty_subjects = SubjectCollection::from_iter([]);
        // The validity, acl, method, and event id do not matter for this test.
        let authorization_backend = event::test_utils::create_authorizer_with_guest_access(false);
        let acl = Acl {
            owner: Access::None,
            moderator: Access::None,
            invited_user: Access::None,
            guest_user: Access::None,
        };
        let admission = authorization_backend
            .apply_acl_for_event(empty_subjects, Get, event::test_utils::EVENT_ID, acl)
            .await
            .unwrap();
        assert_eq!(admission, Admission::AuthenticationRequired);
    }

    #[tokio::test]
    async fn require_authentication_to_apply_acl_for_room() {
        let empty_subjects = SubjectCollection::from_iter([]);
        // The validity, acl, method, and event id do not matter for this test.
        let authorization_backend = event::test_utils::create_authorizer_with_guest_access(false);
        let acl = Acl {
            owner: Access::None,
            moderator: Access::None,
            invited_user: Access::None,
            guest_user: Access::None,
        };
        let admission = authorization_backend
            .apply_acl_for_room(empty_subjects, Get, &room::test_utils::ROOM_ID.into(), acl)
            .await
            .unwrap();
        assert_eq!(admission, Admission::AuthenticationRequired);
    }
}
