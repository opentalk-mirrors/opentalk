// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling events for the `chat` namespace

mod chat_event;
mod error;

pub use chat_event::ChatEvent;
pub use error::ChatError;
