// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling user parameters for the `legal-vote` namespace.

mod allowed_participants;
mod duration;
mod name;
mod subtitle;
mod topic;

pub use allowed_participants::{AllowedParticipants, TooFew};
pub use duration::{Duration, TooShort};
pub use name::Name;
use opentalk_types_common::time::TimeZone;
pub use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};
pub use subtitle::Subtitle;
pub use topic::Topic;

/// The users parameters to start a new vote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserParameters {
    /// On a pseudonymous vote, only the tokens will be published with the results, but not the
    /// participant identities.
    pub pseudonymous: bool,

    /// On a live vote, the interim results are sent to all participants when somebody voted. This
    /// option does not take effect if the vote is pseudonymous.
    pub live: bool,

    /// The name of the vote.
    pub name: Name,

    /// A Subtitle for the vote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<Subtitle>,

    /// The topic that will be voted on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<Topic>,

    /// List of participants that are allowed to cast a vote.
    pub allowed_participants: AllowedParticipants,

    /// Indicates that the `Abstain` vote option is enabled.
    pub enable_abstain: bool,

    /// The vote will automatically stop when every participant voted.
    pub auto_close: bool,

    /// The vote will stop when the duration (in seconds) has passed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<Duration>,

    /// A PDF document will be created when the vote is over.
    pub create_pdf: bool,

    /// An optional timezone, defaults to UTC.
    /// Format as standardized by IANA, e.g.\"CET\" or \"Europe/Vienna\".
    /// See: <https://www.iana.org/time-zones>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<TimeZone>,
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(UserParameters {
            pseudonymous: false,
            live: true,
            name: Name::try_from("Test Name").unwrap(),
            subtitle: Some(Subtitle::try_from("Test Subtitle").unwrap()),
            topic: Some(Topic::try_from("Test Topic").unwrap()),
            allowed_participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)])
                .unwrap(),
            enable_abstain: false,
            auto_close: false,
            duration: Some(Duration::try_from(10).unwrap()),
            create_pdf: false,
            timezone: Some(chrono_tz::Tz::Europe__Berlin.into()),
        })
        .unwrap();

        let expected = json!({
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "subtitle": "Test Subtitle",
            "topic": "Test Topic",
            "allowed_participants": ["00000000-0000-0000-0000-000000000001"],
            "enable_abstain": false,
            "auto_close": false,
            "duration": 10,
            "create_pdf": false,
            "timezone": "Europe/Berlin",
        });

        assert_eq!(produced, expected);

        let produced = serde_json::to_value(UserParameters {
            pseudonymous: false,
            live: true,
            name: Name::try_from("Test Name").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)])
                .unwrap(),
            enable_abstain: false,
            auto_close: false,
            duration: None,
            create_pdf: false,
            timezone: None,
        })
        .unwrap();

        let expected = json!({
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "allowed_participants": ["00000000-0000-0000-0000-000000000001"],
            "enable_abstain": false,
            "auto_close": false,
            "create_pdf": false,
        });

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: UserParameters = serde_json::from_value(json!({
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "subtitle": "Test Subtitle",
            "topic": "Test Topic",
            "allowed_participants": ["00000000-0000-0000-0000-000000000001"],
            "enable_abstain": false,
            "auto_close": false,
            "duration": 10,
            "create_pdf": false,
            "timezone": "Europe/Berlin",

        }))
        .unwrap();

        let expected = UserParameters {
            pseudonymous: false,
            live: true,
            name: Name::try_from("Test Name").unwrap(),
            subtitle: Some(Subtitle::try_from("Test Subtitle").unwrap()),
            topic: Some(Topic::try_from("Test Topic").unwrap()),
            allowed_participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)])
                .unwrap(),
            enable_abstain: false,
            auto_close: false,
            duration: Some(Duration::try_from(10).unwrap()),
            create_pdf: false,
            timezone: Some(chrono_tz::Tz::Europe__Berlin.into()),
        };

        assert_eq!(produced, expected);

        let produced: UserParameters = serde_json::from_value(json!({
            "pseudonymous": false,
            "live": true,
            "name": "Test Name",
            "allowed_participants": ["00000000-0000-0000-0000-000000000001"],
            "enable_abstain": false,
            "auto_close": false,
            "create_pdf": false,

        }))
        .unwrap();

        let expected = UserParameters {
            pseudonymous: false,
            live: true,
            name: Name::try_from("Test Name").unwrap(),
            subtitle: None,
            topic: None,
            allowed_participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)])
                .unwrap(),
            enable_abstain: false,
            auto_close: false,
            duration: None,
            create_pdf: false,
            timezone: None,
        };

        assert_eq!(produced, expected);
    }
}
