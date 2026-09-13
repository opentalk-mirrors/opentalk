// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Response types specific to the OpenTalk v1 API.

mod api_output_link_header;
mod api_response;

pub mod headers;

pub use api_output_link_header::ApiOutputLinkHeader;
pub use api_response::ApiResponse;
