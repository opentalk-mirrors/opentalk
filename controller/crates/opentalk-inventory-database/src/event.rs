// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{collections::BTreeSet, pin::Pin};

use futures_util::Stream;
use opentalk_db_storage as db;
use opentalk_inventory::{
    Event, EventException, EventExceptionId, EventInventory, EventInvite, EventSharedFolder,
    EventTrainingParticipationReportParameterSet, GetEventExceptionsCursor, GetEventsCursor,
    NewEvent, NewEventException, Room, RoomSipConfig, Tariff, UpdateEvent, UpdateEventException,
    User,
};
use opentalk_types_common::{
    events::{EventId, invites::EventInviteStatus},
    rooms::{RoomId, RoomIdOrAlias},
    time::Timestamp,
    training_participation_report::TrainingParticipationReportParameterSet,
    users::UserId,
};
use snafu::ResultExt as _;

use crate::{
    DatabaseConnection, Result, error::DatabaseSnafu, utils::convert_db_stream_to_inventory_stream,
};

#[async_trait::async_trait]
impl EventInventory for DatabaseConnection {
    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_event(&mut self, new_event: NewEvent) -> Result<Event> {
        Ok(
            db::queries::events::create_event(&mut self.inner, new_event.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_event(&mut self, event_id: EventId, event: UpdateEvent) -> Result<Event> {
        Ok(
            db::queries::events::update_event(&mut self.inner, event_id, event.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event(&mut self, event_id: EventId) -> Result<Event> {
        Ok(db::queries::events::get_event(&mut self.inner, event_id)
            .await
            .context(DatabaseSnafu)?
            .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_for_room(
        &mut self,
        room_id_or_alias: &RoomIdOrAlias,
    ) -> Result<Option<Event>> {
        Ok(
            db::queries::events::get_event_for_room(&mut self.inner, room_id_or_alias)
                .await
                .context(DatabaseSnafu)?
                .map(Into::into),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_id_for_room(&mut self, room_id: RoomId) -> Result<Option<EventId>> {
        Ok(
            db::queries::events::get_event_id_for_room(&mut self.inner, room_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_with_room_and_sip_config(
        &mut self,
        event_id: EventId,
    ) -> Result<(Event, Room, Option<RoomSipConfig>)> {
        let (event, room, sip_config) =
            db::queries::events::get_with_room(&mut self.inner, event_id)
                .await
                .context(DatabaseSnafu)?;
        Ok((event.into(), room.into(), sip_config.map(Into::into)))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
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
    )> {
        let (
            event,
            invite,
            room,
            sip_config,
            is_favourite,
            shared_folder,
            tariff,
            training_participation_report_parameter_set,
        ) = db::queries::events::get_with_related_items(&mut self.inner, user_id, event_id)
            .await
            .context(DatabaseSnafu)?;
        Ok((
            event.into(),
            invite.map(Into::into),
            room.into(),
            sip_config.map(Into::into),
            is_favourite,
            shared_folder.map(Into::into),
            tariff.into(),
            training_participation_report_parameter_set.map(Into::into),
        ))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_events_updated_by_user(&mut self, user_id: UserId) -> Result<Vec<Event>> {
        Ok(
            db::queries::events::get_all_events_updated_by_user(&mut self.inner, user_id)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_adhoc_event_and_room_ids_created_before(
        &mut self,
        created_before: Timestamp,
    ) -> Result<Vec<(EventId, RoomId)>> {
        Ok(
            db::queries::events::get_all_event_and_room_ids_adhoc_created_before(
                &mut self.inner,
                created_before.into(),
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_scheduled_event_and_room_ids_ended_before(
        &mut self,
        ended_before: Timestamp,
    ) -> Result<Vec<(EventId, RoomId)>> {
        Ok(
            db::queries::events::get_all_event_and_room_ids_that_ended_before(
                &mut self.inner,
                ended_before.into(),
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_event_and_room_ids_created_by_user_including_disabled(
        &mut self,
        user_id: UserId,
    ) -> Result<Vec<(EventId, RoomId)>> {
        Ok(
            db::queries::events::get_all_events_and_room_ids_for_creator_including_disabled(
                &mut self.inner,
                user_id,
            )
            .await
            .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_finite_recurring_events(&mut self) -> Result<Vec<Event>> {
        Ok(
            db::queries::events::get_all_events_finite_recurring(&mut self.inner)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_all_events_for_user(
        &mut self,
        user: User,
        only_recurring: bool,
    ) -> Result<Vec<Event>> {
        let items = db::queries::events::get_all_events_for_user(
            &mut self.inner,
            user.into(),
            only_recurring,
        )
        .await
        .context(DatabaseSnafu)?;
        Ok(items.into_iter().map(|event| event.into()).collect())
    }

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
    > {
        let stream = db::queries::events::get_all_events_for_user_paginated_as_stream(
            &mut self.inner,
            user.into(),
            only_favorites,
            Vec::from_iter(invite_status_filter),
            time_min.map(Into::into),
            time_max.map(Into::into),
            created_before.map(Into::into),
            created_after.map(Into::into),
            adhoc,
            time_independent,
            cursor.map(Into::into),
        )
        .await
        .context(DatabaseSnafu)?;

        let stream = convert_db_stream_to_inventory_stream(
            stream,
            convert_event_and_related_to_inventory_types,
        )
        .await
        .context(DatabaseSnafu)?;

        Ok(stream)
    }

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
    ) -> Result<Pin<Box<dyn Stream<Item = Result<(EventException, Event)>> + 'a>>> {
        let stream = db::queries::events::get_all_events_exceptions_for_user_paginated_as_stream(
            &mut self.inner,
            user.into(),
            only_favorites,
            Vec::from_iter(invite_status_filter),
            time_min.map(Into::into),
            time_max.map(Into::into),
            created_before.map(Into::into),
            created_after.map(Into::into),
            adhoc,
            time_independent,
            cursor.map(Into::into),
        )
        .await
        .context(DatabaseSnafu)?;

        let stream = convert_db_stream_to_inventory_stream(
            stream,
            convert_event_exception_and_related_to_inventory_types,
        )
        .await
        .context(DatabaseSnafu)?;

        Ok(stream)
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_event_exception(
        &mut self,
        event_exception: NewEventException,
    ) -> Result<EventException> {
        Ok(
            db::queries::events::create_event_exception(&mut self.inner, event_exception.into())
                .await
                .context(DatabaseSnafu)?
                .into(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_exceptions(
        &mut self,
        event_id: EventId,
        timestamps: &[Timestamp],
    ) -> Result<Vec<EventException>> {
        let timestamps: Vec<&_> = timestamps.iter().map(|v| v.as_ref()).collect();
        Ok(
            db::queries::events::get_event_exceptions(&mut self.inner, event_id, &timestamps)
                .await
                .context(DatabaseSnafu)?
                .into_iter()
                .map(Into::into)
                .collect(),
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn get_event_exception(
        &mut self,
        event_id: EventId,
        instance_id_timestamp: Timestamp,
    ) -> Result<Option<EventException>> {
        Ok(db::queries::events::get_event_exception(
            &mut self.inner,
            event_id,
            instance_id_timestamp.into(),
        )
        .await
        .context(DatabaseSnafu)?
        .map(Into::into))
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn update_event_exception(
        &mut self,
        event_exception_id: EventExceptionId,
        event_exception: UpdateEventException,
    ) -> Result<EventException> {
        Ok(db::queries::events::update_event_exception(
            &mut self.inner,
            event_exception.into(),
            event_exception_id.into(),
        )
        .await
        .context(DatabaseSnafu)?
        .into())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_event_exceptions_for_event(&mut self, event_id: EventId) -> Result<()> {
        Ok(
            db::queries::events::delete_event_exceptions_for_event(&mut self.inner, event_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_event_for_room(&mut self, room_id: RoomId) -> Result<()> {
        Ok(
            db::queries::events::delete_event_for_room(&mut self.inner, room_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn create_event_favorite_for_user(
        &mut self,
        event_id: EventId,
        user_id: UserId,
    ) -> Result<bool> {
        let new_event_favorite =
            db::tables::event_favorites::NewEventFavorite { event_id, user_id };

        Ok(db::queries::events::try_create_event_favorite_for_user(
            &mut self.inner,
            new_event_favorite,
        )
        .await
        .context(DatabaseSnafu)?
        .is_some())
    }

    #[tracing::instrument(err(level = "debug"), skip_all)]
    async fn delete_event_favorite_for_user(
        &mut self,
        event_id: EventId,
        user_id: UserId,
    ) -> Result<bool> {
        Ok(
            db::queries::events::delete_event_favorite_for_user(&mut self.inner, user_id, event_id)
                .await
                .context(DatabaseSnafu)?,
        )
    }
}

#[expect(clippy::type_complexity)]
fn convert_event_and_related_to_inventory_types(
    (event, invite, room, sip_config, is_favorite, shared_folder, tariff): (
        db::queries::events::types::EventRecord,
        Option<db::tables::event_invites::EventInvite>,
        db::tables::rooms::Room,
        Option<db::tables::sip_configs::SipConfig>,
        bool,
        Option<db::tables::event_shared_folders::EventSharedFolder>,
        db::tables::tariffs::Tariff,
    ),
) -> (
    Event,
    Option<EventInvite>,
    Room,
    Option<RoomSipConfig>,
    bool,
    Option<EventSharedFolder>,
    Tariff,
    Option<TrainingParticipationReportParameterSet>,
) {
    (
        event.into(),
        invite.map(Into::into),
        room.into(),
        sip_config.map(Into::into),
        is_favorite,
        shared_folder.map(Into::into),
        tariff.into(),
        None, // TODO: remove this (as in opentalk_db_storage::events::get_all_for_user_paginated)
    )
}

fn convert_event_exception_and_related_to_inventory_types(
    (event_exception, event): (
        db::tables::event_exceptions::EventException,
        db::queries::events::types::EventRecord,
    ),
) -> (EventException, Event) {
    (event_exception.into(), event.into())
}
