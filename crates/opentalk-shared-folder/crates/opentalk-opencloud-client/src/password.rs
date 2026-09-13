// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Client-side generation of passwords that conform to an OpenCloud password
//! policy. OpenCloud does not offer a server-side password generator for share
//! links, so a conforming password is built locally from the policy.

use rand::{
    seq::{IndexedRandom as _, SliceRandom},
    Rng,
};

use crate::{types::PasswordPolicy, Error, Result};

const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGITS: &[u8] = b"0123456789";
/// A safe subset of the OWASP special characters, excluding the space, the
/// double quote, the backtick and the backslash to avoid quoting pitfalls.
const SPECIAL: &[u8] = b"!#$%&()*+,-./:;<=>?@[]^_{|}~";

/// The length used when the policy does not request a longer password, to keep
/// generated passwords reasonably strong.
const DEFAULT_MIN_LENGTH: u32 = 16;
/// The maximum length assumed when the policy does not report one.
const FALLBACK_MAX_LENGTH: u32 = 72;

fn push_random(dst: &mut Vec<u8>, set: &[u8], count: u32, rng: &mut impl Rng) {
    for _ in 0..count {
        dst.push(*set.choose(rng).expect("character set is never empty"));
    }
}

/// Generates a random password that satisfies `policy`.
///
/// At least one character of each category (lowercase, uppercase, digit,
/// special) is always included, regardless of the policy, to keep the password
/// strong and avoid trivially banned passwords.
pub(crate) fn generate(policy: &PasswordPolicy) -> Result<String> {
    let mut rng = rand::rng();

    let n_lower = policy.min_lowercase_characters.max(1);
    let n_upper = policy.min_uppercase_characters.max(1);
    let n_digit = policy.min_digits.max(1);
    let n_special = policy.min_special_characters.max(1);
    let required = n_lower + n_upper + n_digit + n_special;

    let max_len = policy.max_characters.unwrap_or(FALLBACK_MAX_LENGTH);

    let mut target = policy.min_characters.max(required).max(DEFAULT_MIN_LENGTH);
    if target > max_len {
        if required > max_len {
            return Err(Error::PasswordPolicyUnsatisfiable);
        }
        target = max_len;
    }

    let mut chars: Vec<u8> = Vec::with_capacity(target as usize);
    push_random(&mut chars, LOWERCASE, n_lower, &mut rng);
    push_random(&mut chars, UPPERCASE, n_upper, &mut rng);
    push_random(&mut chars, DIGITS, n_digit, &mut rng);
    push_random(&mut chars, SPECIAL, n_special, &mut rng);

    let all: Vec<u8> = [LOWERCASE, UPPERCASE, DIGITS, SPECIAL].concat();
    let remaining = target - required;
    push_random(&mut chars, &all, remaining, &mut rng);

    chars.shuffle(&mut rng);

    Ok(String::from_utf8(chars).expect("all characters are ASCII"))
}

#[cfg(test)]
mod tests {
    use super::generate;
    use crate::types::PasswordPolicy;

    fn count(password: &str, set: &[u8]) -> usize {
        password.bytes().filter(|byte| set.contains(byte)).count()
    }

    #[test]
    fn satisfies_a_strict_policy() {
        let policy = PasswordPolicy {
            min_characters: 13,
            max_characters: Some(72),
            min_lowercase_characters: 3,
            min_uppercase_characters: 2,
            min_digits: 2,
            min_special_characters: 2,
        };
        let password = generate(&policy).unwrap();

        assert!(password.chars().count() >= 13);
        assert!(password.chars().count() <= 72);
        assert!(count(&password, super::LOWERCASE) >= 3);
        assert!(count(&password, super::UPPERCASE) >= 2);
        assert!(count(&password, super::DIGITS) >= 2);
        assert!(count(&password, super::SPECIAL) >= 2);
    }

    #[test]
    fn satisfies_a_disabled_policy_with_only_max() {
        let policy = PasswordPolicy {
            max_characters: Some(72),
            ..Default::default()
        };
        let password = generate(&policy).unwrap();

        // Even without minimums, a reasonably strong password is produced.
        assert!(password.chars().count() >= 16);
        assert!(count(&password, super::LOWERCASE) >= 1);
        assert!(count(&password, super::UPPERCASE) >= 1);
        assert!(count(&password, super::DIGITS) >= 1);
        assert!(count(&password, super::SPECIAL) >= 1);
    }

    #[test]
    fn respects_the_maximum_length() {
        let policy = PasswordPolicy {
            min_characters: 100,
            max_characters: Some(20),
            ..Default::default()
        };
        let password = generate(&policy).unwrap();
        assert!(password.chars().count() <= 20);
    }
}
