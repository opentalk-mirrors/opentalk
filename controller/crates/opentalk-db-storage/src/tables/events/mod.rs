// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains events table structs

mod event;
mod event_serial_id;
mod new_event;
mod update_event;

pub use event::Event;
pub use event_serial_id::EventSerialId;
pub use new_event::NewEvent;
pub use update_event::UpdateEvent;
