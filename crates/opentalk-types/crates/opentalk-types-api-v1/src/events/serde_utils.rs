// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::pagination::PageSize;

pub(super) fn invitees_max_or_zero<'de, D>(d: D) -> Result<Option<PageSize>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::{Deserialize, de::Error};

    match Option::<i64>::deserialize(d)? {
        Some(0) | None => Ok(None),
        Some(v) => PageSize::try_from(v).map(Some).map_err(D::Error::custom),
    }
}
