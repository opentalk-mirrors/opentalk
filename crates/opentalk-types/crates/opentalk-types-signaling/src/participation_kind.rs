// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, IntoStaticStr, VariantNames};

use crate::ParticipationVisibility;

/// The kinds of participants in a meeting room.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumCount,
    EnumIter,
    EnumString,
    VariantNames,
    IntoStaticStr,
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(rename_all = "snake_case")
)]
#[cfg_attr(
    feature = "redis",
    derive(redis_args::ToRedisArgs, redis_args::FromRedisValue)
)]
#[cfg_attr(feature = "redis", to_redis_args(Display), from_redis_value(FromStr))]
#[strum(serialize_all = "snake_case")]
pub enum ParticipationKind {
    /// User participation kind is used for regular participants who have an account
    /// in the authentication service.
    User,

    /// Guest participation kind is used for regular participants who do not have
    /// an account in the authentication service.
    Guest,

    /// Sip participation kind is used for participants joining on behalf
    /// of a dial-in user.
    Sip,

    /// Recorder participation kind is used for a participant joining as a
    /// recording service.
    Recorder,
}

impl ParticipationKind {
    /// Returns `true` if the participant kind is visible to other participants
    /// in the room.
    pub fn visibility(&self) -> ParticipationVisibility {
        match self {
            ParticipationKind::User | ParticipationKind::Guest | ParticipationKind::Sip => {
                ParticipationVisibility::Visible
            }
            ParticipationKind::Recorder => ParticipationVisibility::Hidden,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn to_string() {
        assert_eq!(ParticipationKind::Guest.as_ref(), "guest");
        assert_eq!(ParticipationKind::User.as_ref(), "user");
        assert_eq!(ParticipationKind::Sip.as_ref(), "sip");
        assert_eq!(ParticipationKind::Recorder.as_ref(), "recorder");
    }

    #[test]
    fn from_str() {
        assert_eq!(
            ParticipationKind::from_str("guest"),
            Ok(ParticipationKind::Guest)
        );
        assert_eq!(
            ParticipationKind::from_str("user"),
            Ok(ParticipationKind::User)
        );
        assert_eq!(
            ParticipationKind::from_str("sip"),
            Ok(ParticipationKind::Sip)
        );
        assert_eq!(
            ParticipationKind::from_str("recorder"),
            Ok(ParticipationKind::Recorder)
        );
    }

    #[test]
    fn visibility() {
        assert_eq!(
            ParticipationKind::Guest.visibility(),
            ParticipationVisibility::Visible
        );
        assert_eq!(
            ParticipationKind::User.visibility(),
            ParticipationVisibility::Visible
        );
        assert_eq!(
            ParticipationKind::Sip.visibility(),
            ParticipationVisibility::Visible
        );
        assert_eq!(
            ParticipationKind::Recorder.visibility(),
            ParticipationVisibility::Hidden
        );
    }

    #[test]
    fn is_hidden() {
        assert!(!ParticipationKind::Guest.visibility().is_hidden());
        assert!(!ParticipationKind::User.visibility().is_hidden());
        assert!(!ParticipationKind::Sip.visibility().is_hidden());
        assert!(ParticipationKind::Recorder.visibility().is_hidden());
    }

    #[test]
    fn is_visible() {
        assert!(ParticipationKind::Guest.visibility().is_visible());
        assert!(ParticipationKind::User.visibility().is_visible());
        assert!(ParticipationKind::Sip.visibility().is_visible());
        assert!(!ParticipationKind::Recorder.visibility().is_visible());
    }
}
