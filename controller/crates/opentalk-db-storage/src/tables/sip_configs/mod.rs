// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains sip configs table structs

mod new_sip_config;
mod sip_config;
mod update_sip_config;

pub use new_sip_config::NewSipConfig;
pub use sip_config::SipConfig;
pub use update_sip_config::UpdateSipConfig;
