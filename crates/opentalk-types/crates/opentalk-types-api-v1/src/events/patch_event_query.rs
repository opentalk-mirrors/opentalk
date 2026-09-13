// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::pagination::PageSize;

/// Path query parameters for the `PATCH /events/{event_id}` endpoint
#[derive(Default, Debug, Eq, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::IntoParams))]
pub struct PatchEventQuery {
    /// Maximum number of invitees to include inside the event
    #[cfg_attr(
        feature = "serde",
        serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "super::serde_utils::invitees_max_or_zero"
        )
    )]
    pub invitees_max: Option<PageSize>,

    /// Flag to disable email notification
    #[cfg_attr(feature = "serde", serde(default))]
    pub suppress_email_notification: bool,
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::PatchEventQuery;

    #[test]
    fn serialize_default() {
        let example = PatchEventQuery::default();
        assert_eq!(
            json!(example),
            json!({"suppress_email_notification": false})
        );
    }

    #[test]
    fn serialize() {
        let example = PatchEventQuery {
            invitees_max: Some(50.try_into().unwrap()),
            suppress_email_notification: true,
        };
        assert_eq!(
            json!(example),
            json!({"invitees_max": 50, "suppress_email_notification": true})
        );
    }

    #[test]
    fn deserialize_default() {
        let example = PatchEventQuery::default();
        assert_eq!(example, serde_json::from_value(json!({})).unwrap());
    }

    #[test]
    fn deserialize_invitees_max_zero() {
        let example = PatchEventQuery {
            invitees_max: None,
            suppress_email_notification: false,
        };
        assert_eq!(
            example,
            serde_json::from_value(json!({"invitees_max": 0})).unwrap()
        );
    }

    #[test]
    fn deserialize() {
        let example = PatchEventQuery {
            invitees_max: Some(65.try_into().unwrap()),
            suppress_email_notification: true,
        };
        assert_eq!(
            example,
            serde_json::from_value(
                json!({"invitees_max": 65, "suppress_email_notification": true})
            )
            .unwrap()
        );
    }
}
