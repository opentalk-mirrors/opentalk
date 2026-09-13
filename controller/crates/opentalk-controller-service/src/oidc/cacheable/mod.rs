// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Cacheable variants of datatypes.

mod api_error;
mod authentication_error;
mod decode_from_cache_error;
mod error_body;
mod tariff_status;
mod tenant;
mod user;
mod validation_error_entry;

pub use api_error::ApiError;
use authentication_error::AuthenticationError;
pub use decode_from_cache_error::DecodeFromCacheError;
use error_body::ErrorBody;
use tariff_status::TariffStatus;
pub use tenant::Tenant;
pub use user::User;
use validation_error_entry::ValidationErrorEntry;

use super::logout_marker::LogoutMarker;

/// The result of analyzing an access token.
pub type AccessTokenResult = Result<(Tenant, User, LogoutMarker), ApiError>;
