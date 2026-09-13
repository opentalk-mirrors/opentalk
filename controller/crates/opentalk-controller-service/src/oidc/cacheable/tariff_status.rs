// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Archive, Debug, Clone, PartialEq, Eq)]
pub enum TariffStatus {
    Default,
    Paid,
    Downgraded,
}

impl From<opentalk_types_common::tariffs::TariffStatus> for TariffStatus {
    fn from(value: opentalk_types_common::tariffs::TariffStatus) -> Self {
        use opentalk_types_common::tariffs::TariffStatus as Other;

        match value {
            Other::Default => Self::Default,
            Other::Paid => Self::Paid,
            Other::Downgraded => Self::Downgraded,
        }
    }
}

impl From<TariffStatus> for opentalk_types_common::tariffs::TariffStatus {
    fn from(value: TariffStatus) -> Self {
        use TariffStatus as Other;

        match value {
            Other::Default => Self::Default,
            Other::Paid => Self::Paid,
            Other::Downgraded => Self::Downgraded,
        }
    }
}
