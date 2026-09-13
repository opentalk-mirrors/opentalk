// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    tenants::TenantId,
    users::{GroupId, GroupName},
};

/// A group to which users can be associated.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Group {
    /// The id of the group.
    pub id: GroupId,

    /// The name of the group.
    pub name: GroupName,

    /// The id of the tenant to which the group belongs.
    pub tenant_id: TenantId,
}
