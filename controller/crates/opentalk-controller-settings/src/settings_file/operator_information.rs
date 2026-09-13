// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::email::EmailAddress;
use phonenumber::PhoneNumber;
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub(crate) struct OperatorInformation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data_protection_url: Option<Url>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_phone_number: Option<PhoneNumber>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_email_address: Option<EmailAddress>,
}
