// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! Utility to mask phone-number display names of call-in participants.
//!
//! The logic (and the tests below) is ported from the controller's
//! `api/signaling/ws/runner/call_in.rs` so operators see the same masking
//! format they had before signaling was split out of the controller.

use opentalk_types_common::users::DisplayName;
use phonenumber::{Mode, PhoneNumber, country::Id};

/// Mask a call-in participant's display name.
///
/// If the input parses as a phone number (using `default_country_code` as a
/// fallback when the input has no explicit country prefix), the result is the
/// E.164 form with the middle digits replaced by `***`. Otherwise the raw
/// string is masked directly.
pub fn mask_display_name(name: DisplayName, default_country_code: Option<Id>) -> DisplayName {
    let masked = match default_country_code.and_then(|cc| parse_phone_number(name.as_str(), cc)) {
        Some(phone_number) => mask_phone_number(&phone_number),
        None => mask_str(name.as_str()),
    };

    DisplayName::from_str_lossy(&masked)
}

/// Try to parse a phone number and check its validity.
///
/// Returns [`None`] when the phone number is invalid or cannot be parsed.
fn parse_phone_number(phone_number: &str, country_code: Id) -> Option<PhoneNumber> {
    // Strip characters that users commonly include but the parser dislikes.
    let phone_number = phone_number.replace(['(', ')', ' ', '-'], "");

    // The phonenumber crate has some questionable unwraps; catch panics.
    let result =
        std::panic::catch_unwind(move || phonenumber::parse(Some(country_code), phone_number));

    let phone_number = match result {
        Ok(Ok(phone)) => phone,
        Ok(Err(e)) => {
            tracing::debug!("failed to parse phone number: {e}");
            return None;
        }
        Err(e) => {
            tracing::error!("phonenumber crate panicked while parsing phone number: {e:?}");
            return None;
        }
    };

    if !phonenumber::is_valid(&phone_number) {
        return None;
    }

    Some(phone_number)
}

/// Format a [`PhoneNumber`] in E.164, keeping the leading `+` and the first and
/// last three digits visible and replacing the rest with `***`.
fn mask_phone_number(phone_number: &PhoneNumber) -> String {
    let phone_number = phone_number.format().mode(Mode::E164).to_string();
    mask_str(&phone_number)
}

/// Mask a raw string: keep the first four characters when the string starts
/// with `+` (otherwise three) plus the last three characters, and replace the
/// middle with `***`. Shorter inputs keep as many characters as possible.
fn mask_str(phone_number: &str) -> String {
    let len = phone_number.len();
    let start_len = if phone_number.starts_with('+') { 4 } else { 3 }.min(len);
    let end_len = 3.min(len - start_len);

    let start = &phone_number[..start_len];
    let end = &phone_number[len - end_len..];
    format!("{start}***{end}")
}

#[cfg(test)]
mod tests {
    use phonenumber::country::Id;
    use pretty_assertions::assert_eq;

    use super::{mask_phone_number, mask_str};

    #[test]
    fn mask_phone_number_regular() {
        let phone_number = phonenumber::parse(Some(Id::DE), "1234567890").unwrap();
        let produced = mask_phone_number(&phone_number);
        let expected = "+491***890";
        assert_eq!(expected, produced);
    }

    #[test]
    fn mask_phone_number_short() {
        let phone_number = phonenumber::parse(Some(Id::DE), "12").unwrap();
        let produced = mask_phone_number(&phone_number);
        let expected = "+491***2";
        assert_eq!(expected, produced);
    }

    #[test]
    fn mask_phone_number_too_short() {
        let phone_number = "+491";
        let produced = mask_str(phone_number);
        let expected = "+491***";
        assert_eq!(expected, produced);
    }

    #[test]
    fn mask_phone_number_empty() {
        let produced = mask_str("");
        let expected = "***";
        assert_eq!(expected, produced);
    }
}
