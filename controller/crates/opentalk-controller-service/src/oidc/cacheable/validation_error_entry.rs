// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Archive, Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrorEntry {
    pub field: Option<String>,
    pub code: String,
    pub message: Option<String>,
}

impl From<opentalk_types_api_v1::error::ValidationErrorEntry> for ValidationErrorEntry {
    fn from(
        opentalk_types_api_v1::error::ValidationErrorEntry {
            field,
            code,
            message,
        }: opentalk_types_api_v1::error::ValidationErrorEntry,
    ) -> Self {
        ValidationErrorEntry {
            field: field.map(Into::into),
            code: code.into(),
            message: message.map(Into::into),
        }
    }
}

impl From<ValidationErrorEntry> for opentalk_types_api_v1::error::ValidationErrorEntry {
    fn from(
        ValidationErrorEntry {
            field,
            code,
            message,
        }: ValidationErrorEntry,
    ) -> Self {
        opentalk_types_api_v1::error::ValidationErrorEntry {
            field: field.map(Into::into),
            code: code.into(),
            message: message.map(Into::into),
        }
    }
}
