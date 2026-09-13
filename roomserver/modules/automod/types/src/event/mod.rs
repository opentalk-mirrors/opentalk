// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling event messages for the `automod` namespace

mod automod_event;
mod error;
mod stopped_reason;

pub use automod_event::AutomodEvent;
pub use error::AutomodError;
pub use stopped_reason::StoppedReason;
