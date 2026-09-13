// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/services`.

pub mod call_in;
pub mod recording;

mod post_service_start_response_body;

pub use post_service_start_response_body::PostServiceStartResponseBody;
