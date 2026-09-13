// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use diesel::{Identifiable, Queryable};
use opentalk_inventory as inventory;

use crate::{
    schema::jobs,
    tables::jobs::{JobType, SerialJobId},
};

#[derive(Debug, Clone, Queryable, Identifiable, PartialEq, Eq)]
#[diesel(table_name = jobs)]
pub struct Job {
    pub id: SerialJobId,
    pub name: String,
    pub kind: JobType,
    pub parameters: serde_json::Value,
    pub timeout_secs: i32,
    pub recurrence: String,
}

impl From<Job> for inventory::Job {
    fn from(
        Job {
            id,
            name,
            kind,
            parameters,
            timeout_secs,
            recurrence,
        }: Job,
    ) -> Self {
        Self {
            id: id.into(),
            name,
            kind: kind.into(),
            parameters,
            timeout_secs,
            recurrence,
        }
    }
}

impl From<inventory::Job> for Job {
    fn from(
        inventory::Job {
            id,
            name,
            kind,
            parameters,
            timeout_secs,
            recurrence,
        }: inventory::Job,
    ) -> Self {
        Self {
            id: id.into(),
            name,
            kind: kind.into(),
            parameters,
            timeout_secs,
            recurrence,
        }
    }
}
