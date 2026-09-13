// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Definition of error related types used in the OpenTalk API

#[cfg(feature = "http")]
mod api_error;
mod authentication_error;
mod error_body;
mod validation_error_entry;

#[cfg(feature = "http")]
pub use api_error::ApiError;
pub use authentication_error::AuthenticationError;
pub use error_body::ErrorBody;
pub use validation_error_entry::ValidationErrorEntry;

/// Error code when an existing value is ignored
pub const ERROR_CODE_IGNORED_VALUE: &str = "ignored_value";

/// Error code when a required value is missing
pub const ERROR_CODE_VALUE_REQUIRED: &str = "value_required";

/// Error code when an invalid value is encountered
pub const ERROR_CODE_INVALID_VALUE: &str = "invalid_value";
