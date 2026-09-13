// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::num::NonZero;

use crate::settings::settings_file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Internal {
    pub parallel_storage_quota_requests: NonZero<usize>,
}

impl From<settings_file::internal::Internal> for Internal {
    fn from(value: settings_file::internal::Internal) -> Self {
        let settings_file::internal::Internal {
            parallel_storage_quota_requests,
        } = value;

        Self {
            parallel_storage_quota_requests,
        }
    }
}

impl Default for Internal {
    fn default() -> Self {
        Self {
            parallel_storage_quota_requests: NonZero::new(5).expect("5 is not zero"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Internal;

    #[test]
    fn default_does_not_panic() {
        Internal::default();
    }
}
