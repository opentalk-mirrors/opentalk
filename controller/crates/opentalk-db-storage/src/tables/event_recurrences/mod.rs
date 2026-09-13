// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains event dates table structs

mod event_recurrence;
mod new_event_recurrence;
mod update_event_recurrence;

pub use event_recurrence::EventRecurrence;
pub use new_event_recurrence::NewEventRecurrence;
pub use update_event_recurrence::UpdateEventRecurrence;
