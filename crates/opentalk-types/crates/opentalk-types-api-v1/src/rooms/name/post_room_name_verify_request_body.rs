// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::rooms::RoomName;

/// Body for *POST /rooms/name/verify*
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostRoomNameVerifyRequestBody {
    /// The room name
    pub name: RoomName,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use opentalk_types_common::{rooms::RoomName, utils::ExampleData as _};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::PostRoomNameVerifyRequestBody;

    #[test]
    fn serialize() {
        let example = PostRoomNameVerifyRequestBody {
            name: RoomName::example_data(),
        };
        assert_eq!(
            json!(example),
            json!({
                "name": "personal-room-name",
            })
        );
    }

    #[test]
    fn deserialize() {
        let example = PostRoomNameVerifyRequestBody {
            name: RoomName::example_data(),
        };
        assert_eq!(
            example,
            serde_json::from_value::<PostRoomNameVerifyRequestBody>(json!({
                "name": "personal-room-name",
            }))
            .unwrap()
        );
    }
}
