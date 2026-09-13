// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{common::HttpCorsAllowedOrigins, settings_file};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct HttpCors {
    pub allowed_origin: Option<HttpCorsAllowedOrigins>,
}

impl From<settings_file::HttpCors> for HttpCors {
    fn from(settings_file::HttpCors { allowed_origin }: settings_file::HttpCors) -> Self {
        Self { allowed_origin }
    }
}
