// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

/// Verify response body for *POST /rooms/name/verify*
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PostRoomNameVerifyResponseBody {
    /// Whether the room name is available for use
    pub available: bool,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::PostRoomNameVerifyResponseBody;

    #[test]
    fn serialize_response_body() {
        let example = PostRoomNameVerifyResponseBody { available: true };
        assert_eq!(
            json!(example),
            json!({
                "available": true,
            })
        );
    }

    #[test]
    fn deserialize_response_body() {
        let example = PostRoomNameVerifyResponseBody { available: true };
        assert_eq!(
            example,
            serde_json::from_value::<PostRoomNameVerifyResponseBody>(json!({
                "available": true,
            }))
            .unwrap()
        );
    }
}
