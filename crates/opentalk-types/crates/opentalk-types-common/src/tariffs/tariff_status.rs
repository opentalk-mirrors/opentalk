// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::sql_enum;

sql_enum!(
    feature_gated:

    #[derive(PartialEq, Eq)]
    #[cfg_attr(feature="serde", derive(serde::Serialize, serde::Deserialize), serde(rename_all = "snake_case"))]
    #[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
    TariffStatus,
    "tariff_status",
    TariffStatusType,
    {
        Default = b"default",
        Paid = b"paid",
        Downgraded = b"downgraded",
    }
);
