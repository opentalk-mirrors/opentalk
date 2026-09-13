// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{tenants::TenantId, users::GroupName};

use super::Group;
use crate::Result;

/// A trait for retrieving and storing group entities.
#[async_trait::async_trait]
pub trait GroupInventory {
    /// Get or create groups by the group name.
    async fn get_or_create_groups_by_name(
        &mut self,
        groups: &[(TenantId, GroupName)],
    ) -> Result<Vec<Group>>;
}
