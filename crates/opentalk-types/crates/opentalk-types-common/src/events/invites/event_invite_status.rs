// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use core::fmt;
use std::{collections::BTreeSet, str::FromStr};

use crate::{sql_enum, utils::ExampleData};

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(
        feature = "serde",
        derive(serde::Serialize, serde::Deserialize),
        serde(rename_all = "snake_case")
    )]
    #[cfg_attr(
        feature = "utoipa",
        derive(utoipa::ToSchema),
        schema(example = json!(EventInviteStatus::example_data()))
    )]
    EventInviteStatus,
    "event_invite_status",
    EventInviteStatusType,
    {
        Pending = b"pending",
        Accepted = b"accepted",
        Tentative = b"tentative",
        Declined = b"declined",
    }
);

impl FromStr for EventInviteStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "tentative" => Ok(Self::Tentative),
            "declined" => Ok(Self::Declined),
            _ => Err(format!("unknown invite_status {s:?}")),
        }
    }
}

impl fmt::Display for EventInviteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EventInviteStatus::Pending => "pending",
            EventInviteStatus::Accepted => "accepted",
            EventInviteStatus::Tentative => "tentative",
            EventInviteStatus::Declined => "declined",
        })
    }
}

impl EventInviteStatus {
    /// Get all values for this enumeration type
    pub fn all_enum_values() -> BTreeSet<Self> {
        BTreeSet::from_iter([
            Self::Pending,
            Self::Accepted,
            Self::Tentative,
            Self::Declined,
        ])
    }
}

impl ExampleData for EventInviteStatus {
    fn example_data() -> Self {
        Self::Accepted
    }
}
