// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::Display;
use opentalk_inventory as inventory;
use opentalk_types_common::sql_enum;

sql_enum!(
    #[derive(PartialEq, Eq, Display)]
    JobStatus,
    "job_status",
    JobStatusType,
    {
        Started = b"started",
        Succeeded = b"succeeded",
        Failed = b"failed",
    }
);

impl From<JobStatus> for inventory::JobStatus {
    fn from(value: JobStatus) -> Self {
        match value {
            JobStatus::Started => Self::Started,
            JobStatus::Succeeded => Self::Succeeded,
            JobStatus::Failed => Self::Failed,
        }
    }
}

impl From<inventory::JobStatus> for JobStatus {
    fn from(value: inventory::JobStatus) -> Self {
        use inventory::JobStatus as Other;
        match value {
            Other::Started => Self::Started,
            Other::Succeeded => Self::Succeeded,
            Other::Failed => Self::Failed,
        }
    }
}
