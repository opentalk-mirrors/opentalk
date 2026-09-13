// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

pub(crate) mod share_type {
    use crate::ShareType;

    pub fn serialize<S>(
        share_type: &ShareType,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8((*share_type).into())
    }
}

pub(crate) mod optional_share_permissions {
    use std::collections::HashSet;

    use crate::SharePermission;

    pub fn serialize<S>(
        permissions: &Option<HashSet<SharePermission>>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match permissions {
            Some(p) => serializer.serialize_u8(p.iter().fold(0, |flags, p| flags | u8::from(*p))),
            None => serializer.serialize_none(),
        }
    }
}

pub(crate) mod optional_naive_date {
    use chrono::NaiveDate;

    pub fn serialize<S>(
        date: &Option<NaiveDate>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match date {
            Some(d) => serializer.serialize_str(&d.format("%Y-%m-%d").to_string()),
            None => serializer.serialize_none(),
        }
    }
}

pub(crate) mod share_permissions {
    use std::collections::HashSet;

    use serde::Deserialize;

    use crate::SharePermission;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashSet<SharePermission>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: u8 = Deserialize::deserialize(deserializer)?;

        Ok(SharePermission::load_permissions_from_u8(value))
    }

    pub fn serialize<S>(
        permissions: &HashSet<SharePermission>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(permissions.iter().fold(0, |flags, p| flags | u8::from(*p)))
    }
}
