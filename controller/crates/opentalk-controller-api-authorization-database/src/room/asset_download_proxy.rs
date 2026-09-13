// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_api_authorization::authorization::Admission;

use crate::OpenTalkAuthorizerBackend;

impl OpenTalkAuthorizerBackend {
    /// Authorize access to the [`RoomAssetDownloadProxy`] resource
    /// (`GET` on `/rooms/{room_id}/assets/{asset_id}/proxy`).
    ///
    /// Unconditionally allowed: the proxy is gated by the signed
    /// download token supplied in the `token` query parameter. The
    /// endpoint validates the token against the object storage before
    /// streaming any bytes, so no further check is needed at the
    /// middleware level. Modeling it here makes the "allowed by
    /// default" decision explicit and testable.
    ///
    /// | Subject                 | Access |
    /// | ----------------------- | ------ |
    /// | **User**                | rw     |
    /// | **Guest User**          | rw     |
    ///
    /// [`RoomAssetDownloadProxy`]: opentalk_controller_api_authorization::authorization::Resource::RoomAssetDownloadProxy
    pub(crate) const fn authorize_room_asset_download_proxy() -> Admission {
        Admission::Allowed
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use opentalk_controller_api_authorization::authorization::{
        AccessMethod::{self, Get, Post},
        Admission::Allowed,
        AuthorizationTarget, AuthorizerBackend, Resource, Subject, SubjectCollection,
    };
    use opentalk_controller_settings::test_util;
    use opentalk_inventory::MockInventoryProvider;
    use opentalk_types_common::assets::AssetId;
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::{
        OpenTalkAuthorizerBackend,
        event::test_utils::MODULE_FEATURES,
        room::test_utils::{ROOM_ID, USER_ID},
    };

    const ASSET_ID: AssetId = AssetId::from_u128(0x0042);

    #[tokio::test]
    #[rstest]
    #[case::user_get(SubjectCollection::from_iter([Subject::from(USER_ID)]), Get)]
    #[case::user_post(SubjectCollection::from_iter([Subject::from(USER_ID)]), Post)]
    #[case::invite_get(SubjectCollection::from_iter([Subject::Unauthenticated]), Get)]
    #[case::invite_post(SubjectCollection::from_iter([Subject::Unauthenticated]), Post)]
    async fn room_asset_download_proxy_is_unconditionally_allowed(
        #[case] subjects: SubjectCollection,
        #[case] access_method: AccessMethod,
    ) {
        // The authorizer must not consult the inventory.
        let authorizer = OpenTalkAuthorizerBackend::new(
            Arc::new(MockInventoryProvider::new()),
            test_util::settings_provider_from_example_raw_settings(),
            MODULE_FEATURES,
        );

        let admission = authorizer
            .authorize(AuthorizationTarget {
                authenticated_subjects: subjects,
                resource: Resource::RoomAssetDownloadProxy(ROOM_ID.into(), ASSET_ID),
                access_method,
            })
            .await
            .unwrap();
        assert_eq!(
            Allowed, admission,
            "RoomAssetDownloadProxy must always be allowed"
        );
    }
}
