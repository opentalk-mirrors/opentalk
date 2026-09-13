// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use reqwest::StatusCode;
use snafu::Snafu;

use crate::ShareId;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(context(false), display("URL parse error: {source}",))]
    UrlParse { source: url::ParseError },

    #[snafu(context(false), display("Reqwest DAV error: {source}",))]
    ReqwestDav { source: reqwest_dav::Error },

    #[snafu(context(false), display("Reqwest error: {source}",))]
    Reqwest { source: reqwest::Error },

    #[snafu(display("Server returned unauthorized error"))]
    Unauthorized,

    #[snafu(display("Unknown share type"))]
    UnknownShareType,

    #[snafu(display("Public upload was disabled by the admin"))]
    PublicUploadDisabledByAdmin,

    #[snafu(display("File could not be shared"))]
    FileCouldNotBeShared,

    #[snafu(display("Server sent unexpected status code: {status_code}",))]
    UnexpectedStatusCode { status_code: StatusCode },

    #[snafu(display("Wrong or no update parameter given"))]
    WrongParameter,

    #[snafu(display("Share {share_id} not found",))]
    ShareNotFound { share_id: ShareId },

    #[snafu(display("File {file_path} not found",))]
    FileNotFound {
        file_path: String,
        source: reqwest_dav::Error,
    },
}
