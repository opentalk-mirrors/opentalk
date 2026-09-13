// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventDescription, EventTitle},
    time::Timestamp,
    users::UserId,
};

use crate::event::UpdateEventDate;

/// Representation of an update to an [`super::Event`] in the inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEvent {
    /// The title of the event.
    pub title: Option<EventTitle>,

    /// The description of the event.
    pub description: Option<EventDescription>,

    /// The id of the user last updated the event.
    pub updated_by: UserId,

    /// The updated timestamp.
    pub updated_at: Timestamp,

    /// A flag indicating whether this is an ad-hoc event.
    pub is_adhoc: Option<bool>,

    /// A flag indicating whether the details should be shown in the meeting.
    pub show_meeting_details: Option<bool>,

    /// Contains all date related information about the event.
    pub date: Option<UpdateEventDate>,
}
