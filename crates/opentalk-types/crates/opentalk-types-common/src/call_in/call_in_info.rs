// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use super::{CallInId, CallInPassword};
use crate::utils::ExampleData;

/// Information needed to participate in a call-in connection.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "utoipa",
    derive(utoipa::ToSchema),
    schema(example = json!(CallInInfo::example_data()))
)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS), ts(export_to = "common/"))]
pub struct CallInInfo {
    /// SIP Call-In phone number which must be used to reach the room
    pub tel: String,

    /// SIP ID which must transmitted via DTMF (number field on the phone) to identify this room
    pub id: CallInId,

    /// SIP password which must be transmitted via DTMF (number field on the phone) after entering
    /// the `sip_id` to enter the room
    pub password: CallInPassword,

    /// Mask phone numbers of call-in users.
    pub mask_unmapped_phone_numbers: bool,

    /// The default country code (CLDR region codes), which is used if a phone number does not
    /// contain a country code.
    #[cfg_attr(feature = "utoipa", schema(example = "DE"))]
    pub default_country_code: String,
}

impl ExampleData for CallInInfo {
    fn example_data() -> Self {
        Self {
            tel: "+555-123-456-789".to_string(),
            id: "1234567890".parse().expect("valid call-in id"),
            password: "0987654321".parse().expect("valid call-in password"),
            mask_unmapped_phone_numbers: true,
            default_country_code: "DE".to_string(),
        }
    }
}
