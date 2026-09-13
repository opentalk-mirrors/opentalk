// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types used throughout all versions of the OpenTalk API.

mod api_description;
mod open_talk_api;
mod well_known;

pub use api_description::ApiDescription;
pub use open_talk_api::OpenTalkApi;
pub use well_known::WellKnown;
