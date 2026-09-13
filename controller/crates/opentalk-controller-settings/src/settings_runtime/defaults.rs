// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, env};

use icu_locid::{LanguageIdentifier, langid};
use opentalk_types_common::{features::ModuleFeatureId, time::TimeZone};

use crate::{
    SettingsError,
    settings_file::{self, RoomAlias},
};

pub const MIN_ALIAS_SUFFIX_LENGTH: u8 = 8;
pub const MAX_ALIAS_SUFFIX_LENGTH: u8 = 64;

/// Some settings that apply for the whole installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Defaults {
    /// The user language.
    pub user_language: LanguageIdentifier,

    /// The timezone used by the system and as the users' default.
    pub timezone: TimeZone,

    /// A list of disabled features.
    pub disabled_features: BTreeSet<ModuleFeatureId>,

    /// Room alias configuration.
    pub room_alias: RoomAlias,
}

impl TryFrom<settings_file::Defaults> for Defaults {
    type Error = SettingsError;

    fn try_from(
        settings_file::Defaults {
            user_language,
            timezone,
            disabled_features,
            room_alias,
        }: settings_file::Defaults,
    ) -> Result<Self, Self::Error> {
        let room_alias = room_alias.unwrap_or_default();

        if room_alias.suffix_length < MIN_ALIAS_SUFFIX_LENGTH
            || room_alias.suffix_length > MAX_ALIAS_SUFFIX_LENGTH
        {
            Err(SettingsError::InvalidRoomAliasSuffixLength {
                length: room_alias.suffix_length,
            })
        } else {
            Ok(Self {
                user_language: user_language.unwrap_or_else(default_user_language),
                timezone: timezone.unwrap_or_else(global_timezone),
                disabled_features,
                room_alias,
            })
        }
    }
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            user_language: default_user_language(),
            timezone: TimeZone::default(),
            disabled_features: BTreeSet::default(),
            room_alias: RoomAlias::default(),
        }
    }
}

pub(crate) fn default_user_language() -> LanguageIdentifier {
    langid!("en-US")
}

pub(crate) fn global_timezone() -> TimeZone {
    // Take timezone from TZ environment variable or OS configuration, fallback to default
    if let Ok(tz) = env::var("TZ").or_else(|_| iana_time_zone::get_timezone())
        && let Ok(tz) = tz.parse()
    {
        tz
    } else {
        TimeZone::default()
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::{Defaults, MAX_ALIAS_SUFFIX_LENGTH, MIN_ALIAS_SUFFIX_LENGTH};
    use crate::{SettingsError, settings_file};

    fn defaults_with_suffix_length(suffix_length: u8) -> settings_file::Defaults {
        settings_file::Defaults {
            room_alias: Some(settings_file::RoomAlias {
                disable_suffix: false,
                suffix_length,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn default_room_alias_is_valid() {
        let defaults = settings_file::Defaults::default();
        assert!(Defaults::try_from(defaults).is_ok());
    }

    #[test]
    fn suffix_length_below_minimum_is_rejected() {
        let length = MIN_ALIAS_SUFFIX_LENGTH - 1;
        assert_matches!(
            Defaults::try_from(defaults_with_suffix_length(length)),
            Err(SettingsError::InvalidRoomAliasSuffixLength { length: l }) if l == length
        );
    }

    #[test]
    fn suffix_length_above_maximum_is_rejected() {
        let length = MAX_ALIAS_SUFFIX_LENGTH + 1;
        assert!(matches!(
            Defaults::try_from(defaults_with_suffix_length(length)),
            Err(SettingsError::InvalidRoomAliasSuffixLength { length: l }) if l == length
        ));
    }

    #[test]
    fn suffix_length_at_inclusive_boundaries_is_accepted() {
        assert!(Defaults::try_from(defaults_with_suffix_length(MIN_ALIAS_SUFFIX_LENGTH)).is_ok());
        assert!(Defaults::try_from(defaults_with_suffix_length(MAX_ALIAS_SUFFIX_LENGTH)).is_ok());
    }
}
