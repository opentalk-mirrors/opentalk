// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::borrow::Cow;

use super::ValidationErrorEntry;

/// Standard API error body
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorBody {
    /// Machine readable error code
    pub code: Cow<'static, str>,

    /// Human readable message
    pub message: Cow<'static, str>,

    /// Validation errors for unprocessable entities
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub validation_errors: Vec<ValidationErrorEntry>,
}

impl ErrorBody {
    /// Creates a new [`ErrorBody`] without validation errors
    pub fn new(code: impl Into<Cow<'static, str>>, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            validation_errors: vec![],
        }
    }
}
