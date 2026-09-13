// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Data types for handling events.

pub mod invites;

mod event_description;
mod event_id;
mod event_info;
mod event_title;
mod meeting_details;

pub use event_description::EventDescription;
pub use event_id::EventId;
pub use event_info::EventInfo;
pub use event_title::EventTitle;
pub use meeting_details::MeetingDetails;
