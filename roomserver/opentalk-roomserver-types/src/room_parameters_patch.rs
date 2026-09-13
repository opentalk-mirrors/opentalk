// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context;
use opentalk_types_common::{events::EventTitle, rooms::RoomPassword, utils::ExampleData};
use serde::{Deserialize, Deserializer, Serialize};

use crate::room_parameters::RoomParameters;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema), schema(example = json!(RoomParametersPatch::example_data())))]
pub struct RoomParametersPatch {
    #[serde(default, deserialize_with = "some_option")]
    pub password: Option<Option<RoomPassword>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<EventTitle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_meeting_details: Option<bool>,
}

impl ExampleData for RoomParametersPatch {
    fn example_data() -> Self {
        Self {
            password: Some(Some(RoomPassword::example_data())),
            title: Some(EventTitle::example_data()),
            show_meeting_details: Some(true),
        }
    }
}

impl RoomParametersPatch {
    pub fn try_apply(self, room_parameters: &mut RoomParameters) -> anyhow::Result<()> {
        let Self {
            password,
            title,
            show_meeting_details,
        } = self;

        if let Some(password) = password {
            room_parameters.password = password;
        }

        if let Some(show_meeting_details) = show_meeting_details {
            room_parameters.show_meeting_details = show_meeting_details;
        }

        if let Some(title) = title {
            let event = room_parameters
                .event
                .as_mut()
                .context("RoomParameters do not contain an event")?;
            event.title = title;
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        let Self {
            password,
            title,
            show_meeting_details,
        } = self;
        password.is_none() && title.is_none() && show_meeting_details.is_none()
    }
}

/// this ensures that an undefined value is deserialized to `None`, while `null`
/// will be deserialized to `Some(None)`.
///
/// See https://github.com/serde-rs/serde/issues/904#issuecomment-297737140
fn some_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use insta::assert_snapshot;
    use opentalk_types_common::{events::EventTitle, rooms::RoomPassword, utils::ExampleData};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::RoomParametersPatch;
    use crate::room_parameters::RoomParameters;

    #[test]
    fn serialize_room_parameters_patch() {
        let patch = RoomParametersPatch::example_data();
        let raw = serde_json::to_string_pretty(&patch).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "password": "v3rys3cr3t",
          "title": "Team Event",
          "show_meeting_details": true
        }
        "#);
    }

    #[test]
    fn deserialize_room_parameters_patch() {
        let json = json!({
            "password": "v3rys3cr3t",
            "title": "Meeting",
            "show_meeting_details": true
        });
        let produced: RoomParametersPatch = serde_json::from_value(json).unwrap();
        let expected = RoomParametersPatch {
            password: Some(Some(
                RoomPassword::from_str("v3rys3cr3t").expect("Password must be valid"),
            )),
            title: Some(EventTitle::from_str_lossy("Meeting")),
            show_meeting_details: Some(true),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_room_parameters_patch_null_password() {
        let json = json!({
            "password": null,
            "title": "Meeting",
            "show_meeting_details": true
        });
        let produced: RoomParametersPatch = serde_json::from_value(json).unwrap();
        let expected = RoomParametersPatch {
            password: Some(None),
            title: Some(EventTitle::from_str_lossy("Meeting")),
            show_meeting_details: Some(true),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_room_parameters_patch_undefined_password() {
        let json = json!({
            "title": "Meeting",
            "show_meeting_details": true
        });
        let produced: RoomParametersPatch = serde_json::from_value(json).unwrap();
        let expected = RoomParametersPatch {
            password: None,
            title: Some(EventTitle::from_str_lossy("Meeting")),
            show_meeting_details: Some(true),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_no_title() {
        let patch = RoomParametersPatch {
            password: Some(None),
            title: None,
            show_meeting_details: Some(true),
        };
        let raw = serde_json::to_string_pretty(&patch).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "password": null,
          "show_meeting_details": true
        }
        "#);
    }

    #[test]
    fn deserialize_no_title() {
        let json = json!({
            "password": null,
            "show_meeting_details": true
        });
        let produced: RoomParametersPatch = serde_json::from_value(json).unwrap();
        let expected = RoomParametersPatch {
            password: Some(None),
            title: None,
            show_meeting_details: Some(true),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_no_show_meeting_details() {
        let patch = RoomParametersPatch {
            password: Some(None),
            title: Some(EventTitle::from_str_lossy("Meeting")),
            show_meeting_details: None,
        };
        let raw = serde_json::to_string_pretty(&patch).unwrap();

        assert_snapshot!(raw, @r#"
        {
          "password": null,
          "title": "Meeting"
        }
        "#);
    }

    #[test]
    fn deserialize_no_show_meeting_details() {
        let json = json!({
            "title": "Meeting",
            "password": null
        });
        let produced: RoomParametersPatch = serde_json::from_value(json).unwrap();
        let expected = RoomParametersPatch {
            password: Some(None),
            title: Some(EventTitle::from_str_lossy("Meeting")),
            show_meeting_details: None,
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn apply_patch() {
        let mut params = RoomParameters::example_data();
        let password = RoomPassword::from_str("password").unwrap();
        let patch = RoomParametersPatch {
            password: Some(Some(password.clone())),
            title: Some(EventTitle::from_str_lossy("New Title")),
            show_meeting_details: Some(true),
        };

        patch.try_apply(&mut params).expect("Failed to apply patch");

        assert_eq!(params.password.unwrap(), password);
    }
}
