// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Error response types for REST APIv1
use actix_web::{HttpRequest, error::JsonPayloadError};
use opentalk_types_api_v1::error::ApiError;

/// Error handler for the actix JSON extractor
///
/// Gets called when a incoming request results in an [`JsonPayloadError`].
/// Returns a `Bad Request` [`ApiError`] error with an appropriate error code and message.
pub fn json_error_handler(err: JsonPayloadError, _: &HttpRequest) -> actix_web::error::Error {
    let error_code = match err {
        JsonPayloadError::OverflowKnownLength { .. } | JsonPayloadError::Overflow { .. } => {
            "payload_overflow"
        }
        JsonPayloadError::ContentType => "invalid_content_type",
        JsonPayloadError::Deserialize(_) | JsonPayloadError::Serialize(_) => "invalid_json",
        _ => "invalid_payload",
    };
    ApiError::bad_request()
        .with_code(error_code)
        .with_message(err.to_string())
        .into()
}
