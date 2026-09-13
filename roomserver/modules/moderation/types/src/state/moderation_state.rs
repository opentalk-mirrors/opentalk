// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::SignalingModuleFrontendData;
use serde::{Deserialize, Serialize};

use crate::{
    MODERATION_MODULE_ID,
    state::{ChangeDisplayNameRestrictionState, ModeratorJoinInfo},
};

/// The state of the `moderation` module.
///
/// This struct is sent to the participant in the `join_success` message
/// when they join successfully to the meeting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModerationState {
    /// Moderation module data that is only available for moderators
    #[serde(flatten)]
    pub moderator_data: Option<ModeratorJoinInfo>,
    pub display_name_change_restrictions: ChangeDisplayNameRestrictionState,
}

impl SignalingModuleFrontendData for ModerationState {
    const NAMESPACE: Option<ModuleId> = Some(MODERATION_MODULE_ID);
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_moderation_state() {
        let state = ModerationState {
            moderator_data: None,
            display_name_change_restrictions: ChangeDisplayNameRestrictionState::Disabled,
        };

        assert_snapshot!(serde_json::to_string_pretty(&state).unwrap(), @r#"
        {
          "display_name_change_restrictions": {
            "type": "disabled"
          }
        }
        "#);
    }

    #[test]
    fn deserialize_moderation_state() {
        let json = json!({
            "display_name_change_restrictions": {
                "type": "disabled"
            }
        });

        let produced: ModerationState = serde_json::from_value(json).unwrap();
        let expected = ModerationState {
            moderator_data: None,
            display_name_change_restrictions: ChangeDisplayNameRestrictionState::Disabled,
        };

        assert_eq!(produced, expected);
    }
}
