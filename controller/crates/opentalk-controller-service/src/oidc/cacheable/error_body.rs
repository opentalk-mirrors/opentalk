// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use rkyv::{Archive, Deserialize, Serialize};

use super::ValidationErrorEntry;

#[derive(Serialize, Deserialize, Archive, Debug, Clone, PartialEq, Eq)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub validation_errors: Vec<ValidationErrorEntry>,
}

impl From<opentalk_types_api_v1::error::ErrorBody> for ErrorBody {
    fn from(
        opentalk_types_api_v1::error::ErrorBody {
            code,
            message,
            validation_errors,
        }: opentalk_types_api_v1::error::ErrorBody,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            validation_errors: validation_errors.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ErrorBody> for opentalk_types_api_v1::error::ErrorBody {
    fn from(
        ErrorBody {
            code,
            message,
            validation_errors,
        }: ErrorBody,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            validation_errors: validation_errors.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rkyv::{from_bytes, to_bytes};

    use super::ErrorBody;

    #[test]
    fn roundtrip() {
        let raw = opentalk_types_api_v1::error::ErrorBody {
            code: "invalid_token".into(),
            message: "The provided token is invalid".into(),
            validation_errors: vec![opentalk_types_api_v1::error::ValidationErrorEntry {
                field: Some("abc".into()),
                code: "missing".into(),
                message: Some("The field \"abc\" is missing".into()),
            }],
        };

        let encoded = to_bytes::<rkyv::rancor::Error>(&ErrorBody::from(raw.clone())).unwrap();
        let decoded = from_bytes::<ErrorBody, rkyv::rancor::Error>(&encoded)
            .unwrap()
            .into();

        assert_eq!(raw, decoded);
    }
}
