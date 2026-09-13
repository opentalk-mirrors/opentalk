// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_legal_vote::vote::StopKind as TypesStopKind;
use opentalk_types_common::users::UserId;

/// Represents the different reasons a vote can be stopped.
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopKind {
    /// A normal vote stop issued by a user, containing the `UserId` of the issuer.
    ByUser(UserId),

    /// The vote was stopped automatically because all allowed users have voted.
    Auto,

    /// The vote expired after reaching the set duration.
    Expired,
}

impl From<StopKind> for TypesStopKind {
    fn from(value: StopKind) -> Self {
        match value {
            StopKind::ByUser(user_id) => Self::ByUser {
                stopped_by: user_id,
            },
            StopKind::Auto => Self::Auto,
            StopKind::Expired => Self::Expired,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialize_by_user_stop_kind() {
        let produced = serde_json::to_value(StopKind::ByUser(UserId::from_u128(1))).unwrap();

        let expected = json!({
            "by_user": "00000000-0000-0000-0000-000000000001",
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_by_user_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!({
            "by_user": "00000000-0000-0000-0000-000000000001",
        }))
        .unwrap();

        let expected = StopKind::ByUser(UserId::from_u128(1));

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_auto_stop_kind() {
        let produced = serde_json::to_value(StopKind::Auto).unwrap();

        let expected = json!("auto");

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_auto_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!("auto")).unwrap();

        let expected = StopKind::Auto;

        assert_eq!(produced, expected);
    }

    #[test]
    fn serialize_expired_stop_kind() {
        let produced = serde_json::to_value(StopKind::Expired).unwrap();

        let expected = json!("expired");

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialize_expired_stop_kind() {
        let produced: StopKind = serde_json::from_value(json!("expired")).unwrap();

        let expected = StopKind::Expired;

        assert_eq!(produced, expected);
    }
}
