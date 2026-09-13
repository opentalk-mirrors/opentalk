// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::email::EmailAddress;
use phonenumber::PhoneNumber;
use url::Url;

use crate::settings_file;

/// Information regarding the operator that is responsible for user-facing communication and legal
/// disclosure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorInformation {
    /// The URL where users can find the data protection or privacy policy.
    pub data_protection_url: Option<Url>,

    /// The phone number users can call for support.
    pub support_phone_number: Option<PhoneNumber>,

    /// The email address users can contact for support.
    pub support_email_address: Option<EmailAddress>,
}

impl From<settings_file::OperatorInformation> for OperatorInformation {
    fn from(
        settings_file::OperatorInformation {
            data_protection_url,
            support_phone_number,
            support_email_address,
        }: settings_file::OperatorInformation,
    ) -> Self {
        Self {
            data_protection_url,
            support_phone_number,
            support_email_address,
        }
    }
}
