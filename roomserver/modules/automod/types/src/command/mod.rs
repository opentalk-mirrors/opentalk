// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling command messages for the `automod` namespace

mod automod_command;
mod select;

pub use automod_command::AutomodCommand;
pub use select::Select;
