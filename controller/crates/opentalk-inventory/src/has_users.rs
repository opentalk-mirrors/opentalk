// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::users::UserId;

use crate::{Event, EventException};

/// Trait for models that have user-ids attached to them like created_by/updated_by fields
///
/// Used to make batch requests of users after fetching some resources
///
/// Should only be implemented on references of the actual models
pub trait HasUsers {
    /// Populate the destination with the users found in this item.
    fn populate(self, dst: &mut Vec<UserId>);
}

impl<T, I> HasUsers for I
where
    T: HasUsers,
    I: IntoIterator<Item = T>,
{
    fn populate(self, dst: &mut Vec<UserId>) {
        for t in self {
            t.populate(dst);
        }
    }
}

impl HasUsers for &Event {
    fn populate(self, dst: &mut Vec<UserId>) {
        dst.push(self.created_by);
        dst.push(self.updated_by);
    }
}

impl HasUsers for &EventException {
    fn populate(self, dst: &mut Vec<UserId>) {
        dst.push(self.created_by);
    }
}
