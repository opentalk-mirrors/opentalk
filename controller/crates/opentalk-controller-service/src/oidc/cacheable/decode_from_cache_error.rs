// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use snafu::Snafu;

/// An error that is returned when decoding cached values fails
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum DecodeFromCacheError {
    /// Error when parsing a field value fails with an error
    #[snafu(display("Could not parse value in field {field:?} from the cache"))]
    Parse {
        /// The name of the field
        field: &'static str,
        /// The source error that caused the failure
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Error when converting a field value fails without an explicit error
    ///
    /// This error indicates that a conversion did not yield a result, therefore
    /// returned an [`Option::None`].
    #[snafu(display("Could not convert value in field {field:?} from the cache: {message}"))]
    Convert {
        /// The name of the field
        field: &'static str,

        /// A message
        message: String,
    },

    /// Error when reading a HTTP status code stored in the cache
    #[snafu(display(
        "Could read HTTP status code in field {field:?} from the cache, invalid status code {code}"
    ))]
    InvalidHttpStatusCode {
        /// The name of the field
        field: &'static str,

        /// The found HTTP status code number
        code: u16,

        /// The source error that caused the failure
        source: http::status::InvalidStatusCode,
    },
}
