// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::TenantAssignment;
use crate::settings_file;

/// Tenants configuration.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Tenants {
    /// The tenant assignment.
    pub assignment: TenantAssignment,
}

impl From<settings_file::Tenants> for Tenants {
    fn from(settings_file::Tenants { assignment }: settings_file::Tenants) -> Self {
        Self {
            assignment: assignment.map(Into::into).unwrap_or_default(),
        }
    }
}
