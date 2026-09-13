// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashSet;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::AsRefStr,
    strum::Display,
    strum::EnumCount,
    strum::EnumIter,
    strum::EnumString,
    strum::VariantNames,
    strum::IntoStaticStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum SharePermission {
    Read,
    Update,
    Create,
    Delete,
    Share,
}

impl SharePermission {
    pub(crate) fn load_permissions_from_u8(value: u8) -> HashSet<Self> {
        use strum::IntoEnumIterator;
        let mut permissions = HashSet::default();
        for permission in SharePermission::iter() {
            if (u8::from(permission) & value) != 0 {
                permissions.insert(permission);
            }
        }
        permissions
    }
}

impl From<SharePermission> for u8 {
    fn from(value: SharePermission) -> Self {
        match value {
            SharePermission::Read => 0b00000001,
            SharePermission::Update => 0b00000010,
            SharePermission::Create => 0b00000100,
            SharePermission::Delete => 0b00001000,
            SharePermission::Share => 0b00010000,
        }
    }
}
