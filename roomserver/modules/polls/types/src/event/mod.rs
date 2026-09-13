// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types related to signaling events in the `polls` namespace

mod error;
mod polls_event;

pub use error::Error;
pub use polls_event::PollsEvent;
