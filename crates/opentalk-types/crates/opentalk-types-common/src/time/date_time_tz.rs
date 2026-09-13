// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use chrono::{DateTime, TimeZone as _, Utc};

use crate::{time::TimeZone, utils::ExampleData};

/// Representation of a datetime timezone
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(DateTimeTz::example_data())),
)]
pub struct DateTimeTz {
    /// UTC datetime
    pub datetime: DateTime<Utc>,
    /// Timezone in which the datetime was created in
    pub timezone: TimeZone,
}

impl PartialOrd for DateTimeTz {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DateTimeTz {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.datetime.cmp(&other.datetime)
    }
}

impl ExampleData for DateTimeTz {
    fn example_data() -> Self {
        Self {
            datetime: Utc.with_ymd_and_hms(2024, 7, 5, 17, 23, 42).unwrap(),
            timezone: chrono_tz::Europe::Berlin.into(),
        }
    }
}
