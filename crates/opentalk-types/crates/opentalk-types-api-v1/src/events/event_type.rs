// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::utils::ExampleData;

/// Type of event resource.
///
/// Is used as type discriminator in field `type`.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(EventType::example_data()))
)]
pub enum EventType {
    /// Single event
    #[deprecated(
        since = "0.50.0",
        note = "This variant has been encoded in EventResourceDateKind and EventResourceDate."
    )]
    Single,
    /// Recurring event
    #[deprecated(
        since = "0.50.0",
        note = "This variant has been encoded in EventResourceDate."
    )]
    Recurring,
    /// Event instance
    #[deprecated(
        since = "0.53.0",
        note = "This variant has been encoded in EventInstance."
    )]
    Instance,
    /// Event exception
    #[deprecated(
        since = "0.53.0",
        note = "This variant has been encoded in EventExceptionResource."
    )]
    Exception,
}

#[allow(deprecated)]
impl ExampleData for EventType {
    fn example_data() -> Self {
        Self::Instance
    }
}

#[allow(deprecated)]
#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::EventType;

    #[test]
    fn serialize() {
        assert_eq!(
            serde_json::to_value(EventType::Single).unwrap(),
            json!("single")
        );
        assert_eq!(
            serde_json::to_value(EventType::Recurring).unwrap(),
            json!("recurring")
        );
        assert_eq!(
            serde_json::to_value(EventType::Instance).unwrap(),
            json!("instance")
        );
        assert_eq!(
            serde_json::to_value(EventType::Exception).unwrap(),
            json!("exception")
        );
    }

    #[test]
    fn deserialize() {
        assert_eq!(
            serde_json::from_value::<EventType>(json!("single")).unwrap(),
            EventType::Single
        );
        assert_eq!(
            serde_json::from_value::<EventType>(json!("recurring")).unwrap(),
            EventType::Recurring
        );
        assert_eq!(
            serde_json::from_value::<EventType>(json!("instance")).unwrap(),
            EventType::Instance
        );
        assert_eq!(
            serde_json::from_value::<EventType>(json!("exception")).unwrap(),
            EventType::Exception
        );
    }

    #[test]
    fn deserialize_invalid() {
        assert!(serde_json::from_value::<EventType>(json!("okk")).is_err(),);
        assert!(serde_json::from_value::<EventType>(json!("Single")).is_err(),);
        assert!(serde_json::from_value::<EventType>(json!("SINGLE")).is_err(),);
        assert!(serde_json::from_value::<EventType>(json!("")).is_err(),);
    }
}
