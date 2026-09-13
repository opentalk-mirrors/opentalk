// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Definition of results and related helper types used in the OpenTalk API

/// A result type alias for return values from API requests.
#[cfg(feature = "http")]
pub type ApiResult<T, E = crate::error::ApiError> = Result<T, E>;

/// Result types to be returned from actix-web endpoints
#[cfg(feature = "actix")]
pub mod actix {
    use crate::pagination::PagedApiResponse;

    /// A result type alias for JSON return values from API requests.
    pub type JsonApiResult<J, E = crate::error::ApiError> = Result<actix_web::web::Json<J>, E>;

    /// A result type alias for JSON return values from API requests with paging information.
    pub type JsonApiPagedResult<J, P, E = crate::error::ApiError> =
        Result<PagedApiResponse<actix_web::web::Json<J>, P>, E>;

    /// A result type alias for JSON return values from API requests.
    pub type JsonOrOtherApiResult<J, O, E = crate::error::ApiError> =
        Result<actix_web::Either<actix_web::web::Json<J>, O>, E>;
}
