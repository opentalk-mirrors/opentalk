// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::{roomserver::Token, utils::ExampleData};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomServerAccess::example_data())))]
pub struct RoomServerAccess {
    /// The public url for the roomserver with the requested room
    pub public_url: Url,
    /// The signaling token for the requested room
    pub token: Token,
}

impl ExampleData for RoomServerAccess {
    fn example_data() -> Self {
        Self {
            token: Token::example_data(),
            public_url: Url::parse("https://example.com").unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_types_common::roomserver::Token;
    use url::Url;

    use crate::api::roomserver_access::RoomServerAccess;

    #[test]
    fn serialize_room_server_access() {
        let value = serde_json::to_string_pretty(&RoomServerAccess {
            token: Token::nil(),
            public_url: Url::parse("https://example.com/").unwrap(),
        })
        .unwrap();

        assert_snapshot!(value, @r#"
        {
          "public_url": "https://example.com/",
          "token": "00000000-0000-0000-0000-000000000000"
        }
        "#);
    }
}
