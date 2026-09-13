// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use core::fmt;
use std::{collections::HashSet, convert::TryFrom};

use opentalk_types_signaling::ParticipantId;
use serde::{Deserialize, Serialize};

/// Minimum required number of participants.
pub const MIN_PARTICIPANTS: usize = 1;

/// A validated list of allowed participants, ensuring at least one participant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "HashSet<ParticipantId>")]
pub struct AllowedParticipants(HashSet<ParticipantId>);

/// Error when parsing [`AllowedParticipants`].
#[derive(Debug)]
pub struct TooFew {
    /// The minimum length the participant list has to be.
    pub min_length: usize,
}

impl fmt::Display for TooFew {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AllowedParticipants must contain at least {} participant(s).",
            self.min_length
        )
    }
}

impl TryFrom<HashSet<ParticipantId>> for AllowedParticipants {
    type Error = TooFew;

    fn try_from(value: HashSet<ParticipantId>) -> Result<Self, Self::Error> {
        if value.len() >= MIN_PARTICIPANTS {
            Ok(Self(value))
        } else {
            Err(TooFew {
                min_length: MIN_PARTICIPANTS,
            })
        }
    }
}

impl<const N: usize> TryFrom<[ParticipantId; N]> for AllowedParticipants {
    type Error = TooFew;

    fn try_from(value: [ParticipantId; N]) -> Result<Self, Self::Error> {
        let set = HashSet::from_iter(value);
        AllowedParticipants::try_from(set)
    }
}

impl std::ops::Deref for AllowedParticipants {
    type Target = HashSet<ParticipantId>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn allowed_participants_min_length() {
        assert!(
            AllowedParticipants::try_from(HashSet::new()).is_err(),
            "AllowedParticipants must contain at least {MIN_PARTICIPANTS} participant(s)."
        );

        assert!(
            AllowedParticipants::try_from(HashSet::from_iter([ParticipantId::from_u128(1)]))
                .is_ok(),
            "AllowedParticipants should accept a list with at least one participant."
        );
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
    struct TestStruct {
        participants: AllowedParticipants,
    }

    #[test]
    fn serialize_allowed_participants() {
        let produced = serde_json::to_string_pretty(&TestStruct {
            participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)]).unwrap(),
        })
        .unwrap();

        assert_snapshot!(produced, @r#"
        {
          "participants": [
            "00000000-0000-0000-0000-000000000001"
          ]
        }
        "#);
    }

    #[test]
    fn deserialize_allowed_participants() {
        let produced: TestStruct = serde_json::from_value(
            json!({ "participants": HashSet::<ParticipantId>::from_iter([ParticipantId::from_u128(1)]) }),
        )
        .unwrap();
        let expected = TestStruct {
            participants: AllowedParticipants::try_from([ParticipantId::from_u128(1)]).unwrap(),
        };
        assert_eq!(produced, expected);

        let produced: Result<TestStruct, _> = serde_json::from_value(json!({ "participants": [] }));
        assert!(produced.is_err());
    }
}
