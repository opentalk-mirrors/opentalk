// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Headers used for responses to the OpenTalk v1 API.

/// Headers for referencing to related pages.
#[derive(Debug, utoipa::ToSchema)]
#[schema(
    example = "<https://api.example.org/resource?page=2>; rel='next', <https://api.example.org/resource?page=5>; rel='last'"
)]
pub struct PageLink(pub String);

/// Headers for referencing to related cursors.
#[derive(Debug, utoipa::ToSchema)]
#[schema(example = "<https://api.example.org/resource?after=urlencodednextpagetoken>; rel='next'")]
pub struct CursorLink(pub String);
