// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::pagination::PageSize;

/// Path query parameters for the `GET /events/{event_id}` endpoint
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct GetEventQuery {
    /// Maximum number of invitees to return inside the event resource
    ///
    /// Default value is 0
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::serde_utils::invitees_max_or_zero"
        )
    )]
    pub invitees_max: Option<PageSize>,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::GetEventQuery;

    #[test]
    fn serialize_default() {
        let example = GetEventQuery::default();
        assert_eq!(json!(example), json!({}));
    }

    #[test]
    fn serialize() {
        let example = GetEventQuery {
            invitees_max: Some(50.try_into().unwrap()),
        };
        assert_eq!(json!(example), json!({"invitees_max": 50}));
    }

    #[test]
    fn deserialize_default() {
        let example = GetEventQuery::default();
        assert_eq!(example, serde_json::from_value(json!({})).unwrap());
    }

    #[test]
    fn deserialize_invitees_max_zero() {
        let example = GetEventQuery { invitees_max: None };
        assert_eq!(
            example,
            serde_json::from_value(json!({"invitees_max": 0})).unwrap()
        );
    }

    #[test]
    fn deserialize() {
        let example = GetEventQuery {
            invitees_max: Some(65.try_into().unwrap()),
        };
        assert_eq!(
            example,
            serde_json::from_value(json!({"invitees_max": 65})).unwrap()
        );
    }
}
