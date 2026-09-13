// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use reqwest::StatusCode;
use snafu::Snafu;

use crate::PermissionId;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(context(false), display("URL parse error: {source}"))]
    UrlParse { source: url::ParseError },

    #[snafu(context(false), display("Reqwest error: {source}"))]
    Reqwest { source: reqwest::Error },

    #[snafu(display("Server returned unauthorized (401)"))]
    Unauthorized,

    #[snafu(display("Server returned forbidden (403)"))]
    Forbidden,

    #[snafu(display("Server returned bad request (400): {message}"))]
    BadRequest { message: String },

    #[snafu(display("No personal drive was found for the user"))]
    PersonalDriveNotFound,

    #[snafu(display("No item was found at path {path}"))]
    ItemNotFound { path: String },

    #[snafu(display("No folder was found at path {path}"))]
    FolderNotFound { path: String },

    #[snafu(display("Share link {permission_id} not found"))]
    PermissionNotFound { permission_id: PermissionId },

    #[snafu(display("Server sent unexpected status code: {status_code}"))]
    UnexpectedStatusCode { status_code: StatusCode },

    #[snafu(display("The password policy cannot be satisfied"))]
    PasswordPolicyUnsatisfiable,
}
