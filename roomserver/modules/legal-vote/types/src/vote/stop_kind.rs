// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::UserId;
use serde::{Deserialize, Serialize};

/// Represents the different reasons for stopping a vote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "stop_kind")]
pub enum StopKind {
    /// A vote stop issued by a user. Includes the `UserId` of the user who initiated the stop.
    ByUser {
        /// The user who issued the vote to be stopped.
        stopped_by: UserId,
    },

    /// The vote was stopped automatically because all allowed users have voted.
    Auto,

    /// The vote expired after reaching the set duration.
    Expired,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_by_user_stop_kind() {
        let produced = serde_json::to_value(StopKind::ByUser {
            stopped_by: UserId::from_u128(1),
        })
        .unwrap();

        let expected = json!({
            "stop_kind": "by_user",
            "stopped_by": "00000000-0000-0000-0000-000000000001"
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_by_user_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!({
            "stop_kind": "by_user",
            "stopped_by": "00000000-0000-0000-0000-000000000001"
        }))
        .unwrap();

        let expected = StopKind::ByUser {
            stopped_by: UserId::from_u128(1),
        };

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_auto_stop_kind() {
        let produced = serde_json::to_value(StopKind::Auto).unwrap();

        let expected = json!({
            "stop_kind": "auto",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_auto_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!({
            "stop_kind": "auto",
        }))
        .unwrap();

        let expected = StopKind::Auto;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_expired_stop_kind() {
        let produced = serde_json::to_value(StopKind::Expired).unwrap();

        let expected = json!({
            "stop_kind": "expired",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_expired_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!({
            "stop_kind": "expired",
        }))
        .unwrap();

        let expected = StopKind::Expired;

        assert_eq!(produced, expected);
    }
}
