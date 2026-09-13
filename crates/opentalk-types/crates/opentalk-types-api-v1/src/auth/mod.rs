// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/auth`.

pub mod login;

mod get_login_response_body;
mod logout_token;
mod oidc_provider;
mod post_login_response_body;
mod post_logout_request_body;

pub use get_login_response_body::GetLoginResponseBody;
pub use logout_token::LogoutToken;
pub use oidc_provider::OidcProvider;
pub use post_login_response_body::PostLoginResponseBody;
pub use post_logout_request_body::PostLogoutRequestBody;
