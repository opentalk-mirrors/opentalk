// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr;

use serde::Serialize;
use snafu::Snafu;

use super::JsonPatchErrorCode;

#[derive(Debug, Serialize, Snafu)]
#[snafu(display("Json operation failed at index {at_index}: {error_code} - {details} "))]
pub struct JsonPatchError {
    pub error_code: JsonPatchErrorCode,
    /// Human readable error details
    pub details: String,
    /// The index of the of the operation that failed
    pub at_index: usize,
}

impl JsonPatchError {
    pub fn try_from_diesel_error(diesel_error: &diesel::result::Error) -> Option<Self> {
        if let diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            error_info,
        ) = diesel_error
        {
            let error_code = JsonPatchErrorCode::from_str(error_info.message()).ok()?;

            let details = error_info.details()?;

            let at_index = error_info.hint()?.parse().ok()?;

            return Some(JsonPatchError {
                error_code,
                details: details.into(),
                at_index,
            });
        }

        None
    }
}
