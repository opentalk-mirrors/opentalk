// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    users::{UserId, UserInfo},
    utils::ExampleData,
};

/// Public user details.
///
/// Contains general "public" information about a user. Is accessible to all other users.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        PublicUserProfile::example_data()
    )
))]
pub struct PublicUserProfile {
    /// The user id
    pub id: UserId,

    /// The email of the user
    pub email: String,

    /// General information about the user
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub user_info: UserInfo,
}

impl ExampleData for PublicUserProfile {
    fn example_data() -> Self {
        Self {
            id: UserId::from_u128(0xa11c3),
            email: "alice@example.com".to_string(),
            user_info: UserInfo::example_data(),
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod test {
    use opentalk_types_common::utils::ExampleData;
    use serde_json::json;

    use super::PublicUserProfile;

    #[test]
    fn serialize_public_user_profile() {
        let user_profile = PublicUserProfile::example_data();
        let json = json!({
            "id": "00000000-0000-0000-0000-0000000a11c3",
            "email": "alice@example.com",
            "title": "",
            "firstname": "Alice",
            "lastname": "Adams",
            "display_name": "Alice Adams",
            "avatar_url": "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060",
        });

        let value = serde_json::to_value(user_profile).unwrap();

        assert_eq!(json, value);
    }
}
