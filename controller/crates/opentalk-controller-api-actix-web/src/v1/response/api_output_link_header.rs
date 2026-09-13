// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use either::Either;
use opentalk_types_api_v1::pagination::{CursorPagination, PagePagination};

/// Headers for pagination links in API output response.
#[derive(Debug, Clone)]
pub struct ApiOutputLinkHeader {
    /// The pagination
    pub pagination: Option<Either<PagePagination, CursorPagination>>,
}
