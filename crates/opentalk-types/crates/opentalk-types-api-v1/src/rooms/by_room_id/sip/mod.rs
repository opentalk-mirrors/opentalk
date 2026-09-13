// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to the API endpoints under `/rooms/{room_id}/sip`.

mod put_sip_config_request_body;
mod sip_config_resource;

pub use put_sip_config_request_body::PutSipConfigRequestBody;
pub use sip_config_resource::SipConfigResource;
