// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::fmt::Display;

use opentalk_asset_storage::{AssetError, ObjectStorageError};
use opentalk_controller_api_authorization::authorization::{
    AuthorizationChangeError, AuthorizationError,
};
use opentalk_controller_service_facade::StartRoomError;
use opentalk_database::DatabaseError;
use opentalk_types_api_common::error::ApiError;
use rand::distr::uniform;
use snafu::Whatever;

use crate::event::EventRRuleSetError;

/// Wraps an [`ApiError`]. Allows converting foreign error types that are not
/// visible to the [`ApiError`] by using the `?` operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureApiError(pub ApiError);

impl From<CaptureApiError> for ApiError {
    fn from(CaptureApiError(error): CaptureApiError) -> Self {
        error
    }
}

impl From<ApiError> for CaptureApiError {
    fn from(value: ApiError) -> Self {
        CaptureApiError(value)
    }
}

impl<'a> From<&'a CaptureApiError> for &'a ApiError {
    fn from(value: &'a CaptureApiError) -> Self {
        &value.0
    }
}

impl From<DatabaseError> for CaptureApiError {
    fn from(value: DatabaseError) -> Self {
        if value.is_not_found() {
            return CaptureApiError(ApiError::not_found());
        }
        if value.is_unique_violation() {
            log::error!(
                "REST API threw unique violation from Diesel error: {}",
                snafu::Report::from_error(value)
            );
            return CaptureApiError(ApiError::conflict());
        }
        log::error!(
            "REST API threw internal error from Diesel error: {}",
            snafu::Report::from_error(value)
        );
        CaptureApiError(ApiError::internal())
    }
}

impl From<opentalk_inventory::Error> for CaptureApiError {
    fn from(value: opentalk_inventory::Error) -> Self {
        if value.is_not_found() {
            return CaptureApiError(ApiError::not_found());
        }
        if value.is_unique_violation() {
            log::error!(
                "REST API threw unique violation from inventory backend: {}",
                snafu::Report::from_error(value)
            );
            return CaptureApiError(ApiError::conflict());
        }
        if value.is_constraint_violation() {
            return CaptureApiError(ApiError::bad_request());
        }
        log::error!(
            "REST API threw internal error from inventory backend: {}",
            snafu::Report::from_error(value)
        );
        CaptureApiError(ApiError::internal())
    }
}

impl From<diesel::result::Error> for CaptureApiError {
    fn from(value: diesel::result::Error) -> Self {
        Self::from(<diesel::result::Error as Into<DatabaseError>>::into(value))
    }
}

impl From<AuthorizationError> for CaptureApiError {
    fn from(value: AuthorizationError) -> Self {
        log::error!("REST API threw internal error from authorization error: {value}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<AuthorizationChangeError> for CaptureApiError {
    fn from(value: AuthorizationChangeError) -> Self {
        log::error!("REST API threw internal error from authorization change error: {value}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<StartRoomError> for CaptureApiError {
    fn from(value: StartRoomError) -> Self {
        CaptureApiError(ApiError::from(value))
    }
}

impl From<ObjectStorageError> for CaptureApiError {
    fn from(value: ObjectStorageError) -> Self {
        match value {
            ObjectStorageError::SizeLimitExceeded { max_size } => {
                CaptureApiError(ApiError::bad_request().with_message(format!(
                    "Asset size exceeded the configured limit ({max_size} bytes)"
                )))
            }
            other => {
                log::error!("REST API threw internal error from object storage error: {other}");
                CaptureApiError(ApiError::internal())
            }
        }
    }
}

impl From<EventRRuleSetError> for CaptureApiError {
    fn from(value: EventRRuleSetError) -> Self {
        log::error!("REST API threw internal error from event rruleset error: {value}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<opentalk_cache::CacheError> for CaptureApiError {
    fn from(value: opentalk_cache::CacheError) -> Self {
        log::error!("REST API threw internal error while writing/reading cache: {value}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<Whatever> for CaptureApiError {
    fn from(value: Whatever) -> Self {
        let error_report = snafu::Report::from_error(value);
        log::error!("REST API threw generic internal error: {error_report:?}");
        CaptureApiError(ApiError::internal())
    }
}

impl From<AssetError> for CaptureApiError {
    fn from(value: AssetError) -> Self {
        match value {
            err @ AssetError::InventoryConnection { .. }
            | err @ AssetError::InventoryQuery { .. }
            | err @ AssetError::ObjectStorage { .. }
            | err @ AssetError::FileSize { .. }
            | err @ AssetError::Rollback { .. } => {
                log::error!("REST API threw internal error: {err:?}");
                CaptureApiError(ApiError::internal())
            }

            AssetError::AssetStorageExceeded => CaptureApiError(ApiError::storage_quota_exceeded()),
        }
    }
}

impl From<uniform::Error> for CaptureApiError {
    fn from(value: uniform::Error) -> Self {
        log::error!("REST API threw internal error: {value:?}");
        CaptureApiError(ApiError::internal())
    }
}

impl Display for CaptureApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl actix_web::ResponseError for CaptureApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        self.0.status_code()
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        self.0.error_response()
    }
}

/*
impl From<CaptureApiError> for opentalk_inventory::Error {
    fn from(value: CaptureApiError) -> Self {
        value.0.into()
    }
}
*/
