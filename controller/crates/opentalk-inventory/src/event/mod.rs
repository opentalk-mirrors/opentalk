// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_inception)]

mod event;
mod event_date;
mod event_exception;
mod event_exception_id;
mod event_exception_kind;
mod event_inventory;
mod event_recurrence;
mod get_event_exceptions_cursor;
mod get_events_cursor;
mod new_event;
mod new_event_date;
mod new_event_exception;
mod new_event_recurrence;
mod update_event;
mod update_event_date;
mod update_event_exception;
mod update_event_recurrence;

pub use event::Event;
pub use event_date::EventDate;
pub use event_exception::EventException;
pub use event_exception_id::EventExceptionId;
pub use event_exception_kind::EventExceptionKind;
pub use event_inventory::EventInventory;
pub use event_recurrence::EventRecurrence;
pub use get_event_exceptions_cursor::GetEventExceptionsCursor;
pub use get_events_cursor::GetEventsCursor;
pub use new_event::NewEvent;
pub use new_event_date::NewEventDate;
pub use new_event_exception::NewEventException;
pub use new_event_recurrence::NewEventRecurrence;
pub use update_event::UpdateEvent;
pub use update_event_date::UpdateEventDate;
pub use update_event_exception::UpdateEventException;
pub use update_event_recurrence::UpdateEventRecurrence;
