// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Processes phone numbers.

use phonenumber::PhoneNumber;
use snafu::Report;

/// Try to parse a phone number and check its validity
///
/// Returns [`None`] the phone number is invalid or cannot be parsed
pub fn parse_phone_number(
    phone_number: &str,
    country_code: phonenumber::country::Id,
) -> Option<PhoneNumber> {
    // Remove characters from the phone number to make the parsing easier
    // user input may include any of these characters, but may not always be used correctly
    let phone_number = phone_number.replace(['(', ')', ' ', '-'], "");

    // Catch panics because the phonenumber crate has some questionable unwraps
    let result =
        std::panic::catch_unwind(move || phonenumber::parse(Some(country_code), phone_number));

    // check if phonenumber crate panicked or failed to parse
    let phone_number = match result {
        Ok(Ok(phone)) => phone,
        Ok(Err(e)) => {
            log::warn!("failed to parse phone number: {}", Report::from_error(e));
            return None;
        }
        Err(e) => {
            log::error!("phonenumber crate panicked while parsing phone number: {e:?}");
            return None;
        }
    };

    if !phonenumber::is_valid(&phone_number) {
        return None;
    }

    Some(phone_number)
}
