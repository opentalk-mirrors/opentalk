// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    users::{DisplayName, UserTitle},
    utils::ExampleData,
};

/// Information about the room creator
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(
    example = json!(
        UserInfo::example_data()
    )
))]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(serde), from_redis_value(serde))]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export_to = "common/"))]
pub struct UserInfo {
    /// Optional title of the creator
    pub title: UserTitle,

    /// The creators first name
    pub firstname: String,

    /// The creators last name
    pub lastname: String,

    /// The creators display name
    pub display_name: DisplayName,

    /// The creators avatar url
    pub avatar_url: String,
}

impl ExampleData for UserInfo {
    fn example_data() -> Self {
        UserInfo {
            title: "".parse().expect("valid user title"),
            firstname: "Alice".to_string(),
            lastname: "Adams".to_string(),
            display_name: "Alice Adams".parse().expect("valid display name"),
            avatar_url: "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060".to_string(),
        }
    }
}
