// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::TariffStatusMapping;
use crate::settings_file;

/// The dafult tariff name for the static tariff.
pub const DEFAULT_STATIC_TARIFF_NAME: &str = "OpenTalkDefaultTariff";

/// The tariff assignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TariffAssignment {
    /// Static assignment to a tariff name.
    Static {
        /// The assigned static tariff name.
        static_tariff_name: String,
    },
    /// Get the tariff from an external field provided through OIDC.
    ByExternalTariffId {
        /// The description for mapping the status of external tariffs.
        status_mapping: Option<TariffStatusMapping>,
    },
}

impl From<settings_file::TariffAssignment> for TariffAssignment {
    fn from(value: settings_file::TariffAssignment) -> Self {
        match value {
            settings_file::TariffAssignment::Static { static_tariff_name } => Self::Static {
                static_tariff_name: static_tariff_name
                    .unwrap_or_else(|| DEFAULT_STATIC_TARIFF_NAME.to_string()),
            },
            settings_file::TariffAssignment::ByExternalTariffId { status_mapping } => {
                Self::ByExternalTariffId {
                    status_mapping: status_mapping.map(Into::into),
                }
            }
        }
    }
}

impl Default for TariffAssignment {
    fn default() -> Self {
        Self::Static {
            static_tariff_name: DEFAULT_STATIC_TARIFF_NAME.to_string(),
        }
    }
}
