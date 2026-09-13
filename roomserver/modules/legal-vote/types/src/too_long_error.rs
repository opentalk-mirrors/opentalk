// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::fmt;

#[derive(Debug)]
pub struct TooLongError {
    /// The maximum allowed length.
    pub max_length: usize,
}

impl fmt::Display for TooLongError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Value must not be longer than {} characters",
            self.max_length
        )
    }
}
