// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

/// Status of an event
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventStatus::example_data()))
)]
pub enum EventStatus {
    /// Default status, event is ok
    Ok,

    /// Event (or event instance) was cancelled
    Cancelled,
}

impl ExampleData for EventStatus {
    fn example_data() -> Self {
        Self::Ok
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::EventStatus;

    #[test]
    fn serialize() {
        assert_eq!(serde_json::to_value(EventStatus::Ok).unwrap(), json!("ok"));
        assert_eq!(
            serde_json::to_value(EventStatus::Cancelled).unwrap(),
            json!("cancelled")
        );
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            serde_json::from_value::<EventStatus>(json!("ok")).unwrap(),
            EventStatus::Ok
        );
        assert_eq!(
            serde_json::from_value::<EventStatus>(json!("cancelled")).unwrap(),
            EventStatus::Cancelled
        );
    }

    #[test]
    fn deserialize_invalid() {
        assert!(serde_json::from_value::<EventStatus>(json!("okk")).is_err(),);
        assert!(serde_json::from_value::<EventStatus>(json!("")).is_err(),);
    }
}
