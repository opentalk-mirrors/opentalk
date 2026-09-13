// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Provides OpenID Connect stuff.

mod cache;
mod cacheable;
mod claims;
mod introspect_info;
mod jwt;
mod logout_marker;
mod oidc_cache_error;
mod oidc_context;
mod oidc_token_handler;
mod open_id_connect_user_info;
mod provider;
mod realm_roles;
mod verification_info;

pub use cache::Cache;
pub use cacheable::AccessTokenResult;
pub use claims::OnlyExpiryClaim;
use claims::{JWTAccessTokenClaims, JWTLogoutTokenClaims, OpenTalkAdditionalClaims, ServiceClaims};
use introspect_info::IntrospectInfo;
pub use jwt::{VerifyError, decode_token};
pub use logout_marker::LogoutMarker;
pub use oidc_cache_error::OidcCacheError;
use oidc_context::OidcContext;
pub use oidc_token_handler::{OidcTokenHandler, build_oidc_token_handler};
pub use open_id_connect_user_info::OpenIdConnectUserInfo;
use provider::ProviderClient;
pub use realm_roles::RealmRoles;
use verification_info::VerificationInfo;
