// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, pin::Pin};

use futures_util::Stream;
use opentalk_types_common::{
    events::{EventId, invites::EventInviteStatus},
    rooms::{RoomId, RoomIdOrAlias},
    time::Timestamp,
    training_participation_report::TrainingParticipationReportParameterSet,
    users::UserId,
};

use super::{
    Event, EventException, EventExceptionId, GetEventsCursor, NewEvent, NewEventException,
    UpdateEvent, UpdateEventException,
};
use crate::{
    EventInvite, EventSharedFolder, EventTrainingParticipationReportParameterSet,
    GetEventExceptionsCursor, Result, Room, RoomSipConfig, Tariff, User,
};

/// A trait for retrieving and storing event entities.
#[async_trait::async_trait]
pub trait EventInventory {
    /// Create an event.
    async fn create_event(&mut self, new_event: NewEvent) -> Result<Event>;

    /// Update an event.
    async fn update_event(&mut self, event_id: EventId, event: UpdateEvent) -> Result<Event>;

    /// Get an event by the event id.
    async fn get_event(&mut self, event_id: EventId) -> Result<Event>;

    /// Get an event by the room id.
    async fn get_event_for_room(
        &mut self,
        room_id_or_alias: &RoomIdOrAlias,
    ) -> Result<Option<Event>>;

    /// Get an event id by the room id.
    async fn get_event_id_for_room(&mut self, room_id: RoomId) -> Result<Option<EventId>>;

    /// Get an event with the corresponding room and optional SIP config by the event id.
    async fn get_event_with_room_and_sip_config(
        &mut self,
        event_id: EventId,
    ) -> Result<(Event, Room, Option<RoomSipConfig>)>;

    /// Get an event with related items.
    async fn get_event_with_related_items(
        &mut self,
        user_id: UserId,
        event_id: EventId,
    ) -> Result<(
        Event,
        Option<EventInvite>,
        Room,
        Option<RoomSipConfig>,
        bool,
        Option<EventSharedFolder>,
        Tariff,
        Option<EventTrainingParticipationReportParameterSet>,
    )>;

    /// Get all events updated by a specific user.
    async fn get_all_events_updated_by_user(&mut self, user: UserId) -> Result<Vec<Event>>;

    /// Get the ids of all ad-hoc events that were created before a timestamp.
    async fn get_all_adhoc_event_and_room_ids_created_before(
        &mut self,
        created_before: Timestamp,
    ) -> Result<Vec<(EventId, RoomId)>>;

    /// Get all scheduled events that ended before a specific timestamp.
    async fn get_all_scheduled_event_and_room_ids_ended_before(
        &mut self,
        ended_before: Timestamp,
    ) -> Result<Vec<(EventId, RoomId)>>;

    /// Get the event ids for all events created by a specific user.
    async fn get_all_event_and_room_ids_created_by_user_including_disabled(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<(EventId, RoomId)>>;

    /// Get all finite recurring events.
    async fn get_all_finite_recurring_events(&mut self) -> Result<Vec<Event>>;

    /// Get all events to which a user has access.
    async fn get_all_events_for_user(
        &mut self,
        user: User,
        only_recurring: bool,
    ) -> Result<Vec<Event>>;

    /// Get all events to which a user has access.
    #[expect(clippy::too_many_arguments)]
    async fn get_all_events_for_user_paginated_as_stream<'a>(
        &'a mut self,
        user: User,
        only_favorites: bool,
        invite_status_filter: BTreeSet<EventInviteStatus>,
        time_min: Option<Timestamp>,
        time_max: Option<Timestamp>,
        created_before: Option<Timestamp>,
        created_after: Option<Timestamp>,
        adhoc: Option<bool>,
        time_independent: Option<bool>,
        cursor: Option<GetEventsCursor>,
    ) -> Result<
        Pin<
            Box<
                dyn Stream<
                        Item = Result<(
                            Event,
                            Option<EventInvite>,
                            Room,
                            Option<RoomSipConfig>,
                            bool,
                            Option<EventSharedFolder>,
                            Tariff,
                            Option<TrainingParticipationReportParameterSet>,
                        )>,
                    > + 'a,
            >,
        >,
    >;

    /// Get all event exceptions to which a user has access.
    #[expect(clippy::too_many_arguments)]
    async fn get_all_event_exceptions_for_user_paginated_as_stream<'a>(
        &'a mut self,
        user: User,
        only_favorites: bool,
        invite_status_filter: BTreeSet<EventInviteStatus>,
        time_min: Option<Timestamp>,
        time_max: Option<Timestamp>,
        created_before: Option<Timestamp>,
        created_after: Option<Timestamp>,
        adhoc: Option<bool>,
        time_independent: Option<bool>,
        cursor: Option<GetEventExceptionsCursor>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<(EventException, Event)>> + 'a>>>;

    /// Create an event exception.
    async fn create_event_exception(
        &mut self,
        event_exception: NewEventException,
    ) -> Result<EventException>;

    /// Get a defined list of exceptions for an event.
    async fn get_event_exceptions(
        &mut self,
        event_id: EventId,
        timestamps: &[Timestamp],
    ) -> Result<Vec<EventException>>;

    /// Get an event exception for an event.
    async fn get_event_exception(
        &mut self,
        event_id: EventId,
        instance_id_timestamp: Timestamp,
    ) -> Result<Option<EventException>>;

    /// Update an event exception.
    async fn update_event_exception(
        &mut self,
        event_exception_id: EventExceptionId,
        event_exception: UpdateEventException,
    ) -> Result<EventException>;

    /// Delete all event exceptions for an event.
    async fn delete_event_exceptions_for_event(&mut self, event_id: EventId) -> Result<()>;

    /// Delete the event by its room id.
    async fn delete_event_for_room(&mut self, room_id: RoomId) -> Result<()>;

    /// Create an event favorite.
    async fn create_event_favorite_for_user(
        &mut self,
        event_id: EventId,
        user_id: UserId,
    ) -> Result<bool>;

    /// Delete an event favorite for a user.
    ///
    /// Returns true if something was deleted
    async fn delete_event_favorite_for_user(
        &mut self,
        event_id: EventId,
        user_id: UserId,
    ) -> Result<bool>;
}
