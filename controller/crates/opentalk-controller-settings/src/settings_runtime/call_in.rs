// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::settings_file;

/// Call-in settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallIn {
    /// The call-in telephone number.
    pub tel: String,

    /// Mask unmapped phone numbers to protect privacy.
    pub mask_unmapped_numbers: bool,

    /// The default country code.
    pub default_country_code: phonenumber::country::Id,
}

impl CallIn {
    /// Returns a reference to the tel of this [`CallIn`].
    pub fn tel(&self) -> &str {
        &self.tel
    }

    /// Returns the mask unmapped numbers of this [`CallIn`].
    pub fn mask_unmapped_numbers(&self) -> bool {
        self.mask_unmapped_numbers
    }

    /// Returns the default country code of this [`CallIn`].
    pub fn default_country_code(&self) -> phonenumber::country::Id {
        self.default_country_code
    }
}

impl From<settings_file::CallIn> for CallIn {
    fn from(
        settings_file::CallIn {
            tel,
            mask_unmapped_numbers,
            default_country_code,
        }: settings_file::CallIn,
    ) -> Self {
        Self {
            tel,
            mask_unmapped_numbers,
            default_country_code,
        }
    }
}
