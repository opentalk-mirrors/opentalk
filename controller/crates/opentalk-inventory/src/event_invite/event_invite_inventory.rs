// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::{
    events::{EventId, invites::EventInviteStatus},
    pagination::{ItemCount, Page, PageSize},
    rooms::RoomId,
    users::UserId,
};

use super::{
    EventEmailInvite, NewEventEmailInvite, NewEventInvite, UpdateEventEmailInvite,
    UpdateEventInvite,
};
use crate::{Event, EventInvite, Result, User};

/// A trait for retrieving and storing event invite entities.
#[async_trait::async_trait]
pub trait EventInviteInventory {
    /// Attempts to create an [`EventEmailInvite`]. Returns `Ok(None)` when
    /// an invitation for the same address already exists in the event.
    async fn try_create_event_email_invite(
        &mut self,
        invite: NewEventEmailInvite,
    ) -> Result<Option<EventEmailInvite>>;

    /// Attempts to create an [`EventInvite`]. Returns `Ok(None)` when an
    /// invitation for the same user already exists in the event.
    async fn try_create_event_invite(
        &mut self,
        invite: NewEventInvite,
    ) -> Result<Option<EventInvite>>;

    /// Get the invites by event id, paginated.
    async fn get_event_invites_paginated(
        &mut self,
        event_id: EventId,
        per_page: PageSize,
        page: Page,
        filter_by_status: Option<EventInviteStatus>,
    ) -> Result<(Vec<(EventInvite, User)>, ItemCount)>;

    /// Get the E-Mail invites by event id, paginated.
    async fn get_event_email_invites_paginated(
        &mut self,
        event_id: EventId,
        per_page: PageSize,
        page: Page,
    ) -> Result<(Vec<EventEmailInvite>, ItemCount)>;

    /// Get an event invite to a room for a specific user.
    async fn get_event_invite_for_user_and_room(
        &mut self,
        user_id: UserId,
        room_id: RoomId,
    ) -> Result<Option<EventInvite>>;

    /// Get event user invites for a set of events.
    async fn get_event_user_invites_for_events(
        &mut self,
        events: &[&Event],
    ) -> Result<Vec<Vec<(EventInvite, User)>>>;

    /// Get event email invites for a set of events.
    async fn get_event_email_invites_for_events(
        &mut self,
        events: &[&Event],
    ) -> Result<Vec<Vec<EventEmailInvite>>>;

    /// Get all pending invites for a user.
    async fn get_invites_pending_for_user(&mut self, user_id: UserId) -> Result<Vec<EventInvite>>;

    /// Delete an event invite for a specific invitee.
    async fn delete_event_invite_by_invitee(
        &mut self,
        event_id: EventId,
        user_id: UserId,
    ) -> Result<EventInvite>;

    /// Delete an event invite for an E-Mail address.
    async fn delete_event_invite_by_email(
        &mut self,
        event_id: EventId,
        email: &str,
    ) -> Result<EventEmailInvite>;

    /// Update a user event invite.
    async fn update_event_user_invite(
        &mut self,
        event_id: EventId,
        user_id: UserId,
        event_invite: UpdateEventInvite,
    ) -> Result<EventInvite>;

    /// Update an email event invite.
    async fn update_event_email_invite(
        &mut self,
        event_id: EventId,
        email: &str,
        event_invite: UpdateEventEmailInvite,
    ) -> Result<EventEmailInvite>;

    /// Migrate event email invites to event user invites.
    async fn migrate_event_email_invites_to_user_invites(
        &mut self,
        user: User,
    ) -> Result<Vec<(EventId, RoomId)>>;
}
