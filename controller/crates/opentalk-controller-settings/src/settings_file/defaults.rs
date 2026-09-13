// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::BTreeSet;

use icu_locid::LanguageIdentifier;
use opentalk_types_common::{features::ModuleFeatureId, time::TimeZone};
use serde::Deserialize;

use super::RoomAlias;

#[derive(Clone, Default, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct Defaults {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_language: Option<LanguageIdentifier>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<TimeZone>,

    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub disabled_features: BTreeSet<ModuleFeatureId>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_alias: Option<RoomAlias>,
}
