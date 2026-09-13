// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::Queryable;
use opentalk_types_common::users::{GroupId, UserId};

use crate::{
    schema::user_groups,
    tables::{groups::Group, users::User},
};

#[derive(Debug, Queryable, Identifiable, Associations)]
#[diesel(table_name = user_groups)]
#[diesel(belongs_to(User, foreign_key = user_id))]
#[diesel(belongs_to(Group, foreign_key = group_id))]
#[diesel(primary_key(user_id, group_id))]
pub struct UserGroupRelation {
    pub user_id: UserId,
    pub group_id: GroupId,
}
