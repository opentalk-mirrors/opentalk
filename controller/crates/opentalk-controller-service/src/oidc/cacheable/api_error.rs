// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_controller_utils::CaptureApiError;
use rkyv::{Archive, Deserialize, Serialize};
use snafu::ResultExt as _;

use super::{
    AuthenticationError, DecodeFromCacheError, ErrorBody,
    decode_from_cache_error::InvalidHttpStatusCodeSnafu,
};

/// (De)Serializable version of [`opentalk_types_api_v1::error::ApiError`] so it can be externally cached
#[derive(Serialize, Deserialize, Archive, Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    status: u16,
    www_authenticate: Option<AuthenticationError>,
    body: ErrorBody,
}

impl From<opentalk_types_api_v1::error::ApiError> for ApiError {
    fn from(
        opentalk_types_api_v1::error::ApiError {
            status,
            www_authenticate,
            body,
        }: opentalk_types_api_v1::error::ApiError,
    ) -> Self {
        Self {
            status: status.as_u16(),
            www_authenticate: www_authenticate.map(Into::into),
            body: body.into(),
        }
    }
}

impl TryFrom<ApiError> for opentalk_types_api_v1::error::ApiError {
    type Error = DecodeFromCacheError;

    fn try_from(
        ApiError {
            status,
            www_authenticate,
            body,
        }: ApiError,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            status: status.try_into().context(InvalidHttpStatusCodeSnafu {
                field: "status",
                code: status,
            })?,
            www_authenticate: www_authenticate.map(Into::into),
            body: body.into(),
        })
    }
}

impl From<CaptureApiError> for ApiError {
    fn from(value: CaptureApiError) -> Self {
        Self::from(opentalk_types_api_v1::error::ApiError::from(value))
    }
}

impl TryFrom<ApiError> for CaptureApiError {
    type Error = DecodeFromCacheError;

    fn try_from(value: ApiError) -> Result<Self, Self::Error> {
        opentalk_types_api_v1::error::ApiError::try_from(value).map(Into::into)
    }
}
