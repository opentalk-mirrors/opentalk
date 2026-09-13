// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::{GroupId, UserId};

use crate::schema::user_groups;

#[derive(Debug, Insertable)]
#[diesel(table_name = user_groups)]
pub struct NewUserGroupRelation {
    pub user_id: UserId,
    pub group_id: GroupId,
}
