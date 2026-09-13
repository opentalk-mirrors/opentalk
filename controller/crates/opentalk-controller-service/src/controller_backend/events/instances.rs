// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event instances

use std::{cmp::Ordering, collections::BTreeSet, pin::Pin, sync::Arc};

use async_stream::stream;
use chrono::{DateTime, Duration, Utc};
use futures::pin_mut;
use futures_core::Stream;
use futures_util::{StreamExt, TryStreamExt};
use opentalk_controller_service_facade::RequestUser;
use opentalk_controller_settings::Settings;
use opentalk_controller_utils::{
    CaptureApiError,
    event::{EventDateExt, EventExt},
};
use opentalk_inventory::{
    Event, EventDate, EventException, EventExceptionKind, EventInvite, EventSharedFolder,
    GetEventsCursor, InventoryProvider, NewEventException, Room, RoomSipConfig, Tariff,
    UpdateEventException, User,
};
use opentalk_types_api_v1::{
    error::ApiError,
    events::{
        EventAndInstanceId, EventInstance, EventInstancePath, EventInstanceQuery, EventInvitee,
        EventOrInstance, EventRoomInfo, EventStatus, GetEventInstanceResponseBody,
        GetEventInstancesCursorData, GetEventInstancesQuery, GetEventInstancesResponseBody,
        GetEventsAndInstancesQuery, GetEventsCursorData, InstanceId, InstanceMarker,
        PatchEventInstanceBody,
    },
    pagination::Cursor,
};
use opentalk_types_common::{
    events::{EventId, invites::EventInviteStatus},
    pagination::PageSize,
    shared_folders::SharedFolder,
    time::{DateTimeTz, TimeZone, Timestamp},
    training_participation_report::TrainingParticipationReportParameterSet,
    users::Language,
};

use crate::{
    ControllerBackend,
    controller_backend::{
        events::{DateTimeTzFromInventory, EventRoomInfoExt, ONE_HUNDRED_YEARS_IN_DAYS},
        utils::interweave_result_streams,
    },
    events::{
        enrich_invitees_from_optional_user_search, get_invited_mail_recipients_for_event,
        notifications::{UpdateNotificationValues, notify_invitees_about_update},
        shared_folder_for_user,
    },
    user_profiles::{GetUserProfilesBatched, UserProfilesBatch},
};

const ONE_HUNDRED_YEARS: Duration = Duration::days(ONE_HUNDRED_YEARS_IN_DAYS as i64);

#[expect(clippy::type_complexity)]
enum InternalEventOrInstance {
    Event(
        (
            Event,
            Option<EventInvite>,
            Box<Room>,
            Option<RoomSipConfig>,
            bool,
            Option<EventSharedFolder>,
            Tariff,
            Option<TrainingParticipationReportParameterSet>,
        ),
    ),
    Instance(EventInstance),
}

impl ControllerBackend {
    pub(crate) async fn get_events_and_instances_interwoven(
        &self,
        current_user: RequestUser,
        query: GetEventsAndInstancesQuery,
    ) -> Result<(Vec<EventOrInstance>, Option<String>, Option<String>), CaptureApiError> {
        let get_events_cursor = query.after.map(|cursor| {
            GetEventsCursor::new(
                cursor.event_id,
                cursor.event_created_at,
                cursor.event_starts_at,
            )
        });

        let mut inventory = self.inventory_provider.get_inventory().await?;
        let current_user = inventory.get_user(current_user.id).await?;

        let recurring_events = match query.time_independent {
            Some(false) | None => {
                // Get all recurring events we intend to get instances for
                inventory
                    .get_all_events_for_user(current_user.clone(), true)
                    .await?
            }
            // Omit recurring events for time independent query
            Some(true) => Vec::new(),
        };

        // Get the events and belonging data as a stream that can be interwoven
        let events_stream = inventory
            .get_all_events_for_user_paginated_as_stream(
                current_user.clone(),
                query.favorites,
                BTreeSet::from_iter(query.invite_status.clone()),
                query.time_min,
                query.time_max,
                query.created_before,
                query.created_after,
                query.adhoc,
                query.time_independent,
                get_events_cursor,
            )
            .await?;

        // Return all non-recurring events, but recurring events only if event instances are suppressed
        let events_stream = events_stream.filter(|result| {
            futures::future::ready({
                if let Ok(stream_item) = result {
                    !stream_item.0.is_recurring() || query.instances_max.is_zero()
                } else {
                    true
                }
            })
        });

        // Prepare all streams to interweave (one for the events and one for each event's instances).
        let mut events_and_instances_streams: Vec<
            Pin<Box<dyn Stream<Item = opentalk_inventory::Result<InternalEventOrInstance>>>>,
        > = Vec::with_capacity(recurring_events.len() + 1);
        events_and_instances_streams.push(Box::pin(events_stream.map_ok(
            |(event, invite, room, sip_config, is_favorite, shared_folder, tariff, training)| {
                InternalEventOrInstance::Event((
                    event,
                    invite,
                    Box::new(room),
                    sip_config,
                    is_favorite,
                    shared_folder,
                    tariff,
                    training,
                ))
            },
        )));
        for event in recurring_events {
            let cursor_event_starts_at = query.after.and_then(|cursor| cursor.event_starts_at);

            let (time_min, time_min_inclusive) = match (query.time_min, cursor_event_starts_at) {
                (v, None) => (v, true),
                (None, v) => (v, false),
                (Some(v1), Some(v2)) => {
                    if v1 > v2 {
                        (Some(v1), true)
                    } else {
                        (Some(v2), false)
                    }
                }
            };

            let instances_stream = self
                .get_event_instances_as_stream(
                    &current_user,
                    event.id,
                    query.invitees_max,
                    time_min,
                    time_min_inclusive,
                    query.time_max,
                    query.instances_max.into(),
                    0,
                )
                .await?;

            events_and_instances_streams.push(Box::pin(
                instances_stream.map_ok(InternalEventOrInstance::Instance),
            ));
        }

        // Interweave the streams
        let stream = interweave_result_streams(
            events_and_instances_streams,
            compare_inventory_events_or_instances,
        );

        // Create the resource vector
        let settings = self.settings_provider.get();
        let (vector, has_more_items) = self
            .create_events_or_instances_vec_from_stream(
                self.inventory_provider.clone(),
                &settings,
                current_user.clone(),
                stream,
                query.invitees_max,
                query.per_page,
            )
            .await?;

        let after = build_after_cursor(&vector, has_more_items);
        let before = None;

        Ok((vector, before, after))
    }

    async fn create_events_or_instances_vec_from_stream(
        &self,
        inventory_provider: Arc<dyn InventoryProvider>,
        settings: &Settings,
        current_user: User,
        stream: impl Stream<Item = opentalk_inventory::Result<InternalEventOrInstance>>,
        invitees_max: Option<PageSize>,
        per_page: Option<PageSize>,
    ) -> Result<(Vec<EventOrInstance>, bool), CaptureApiError> {
        let limit: usize = per_page.unwrap_or_default().into();

        let (raw_items, has_more_items) = take_with_has_more(stream, limit).await?;

        let mut inventory = inventory_provider.get_inventory().await?;
        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;

        let mut items: Vec<EventOrInstance> = Vec::with_capacity(raw_items.len());
        for raw in raw_items {
            let item = match raw {
                InternalEventOrInstance::Event((
                    event,
                    invite,
                    room,
                    sip_config,
                    is_favorite,
                    shared_folder,
                    tariff,
                    training,
                )) => {
                    let resource = self
                        .build_event_resource(
                            inventory.as_mut(),
                            settings,
                            current_user.clone(),
                            &current_tenant,
                            invitees_max,
                            (
                                event,
                                invite,
                                *room,
                                sip_config,
                                is_favorite,
                                shared_folder,
                                tariff,
                                training,
                            ),
                        )
                        .await?;
                    EventOrInstance::Event(resource)
                }
                InternalEventOrInstance::Instance(instance) => {
                    let enriched_invitees = enrich_invitees_from_optional_user_search(
                        settings,
                        &self.user_search_client,
                        &current_tenant,
                        instance.invitees.clone(),
                    )
                    .await;

                    let instance = EventInstance {
                        invitees: enriched_invitees,
                        ..instance
                    };

                    EventOrInstance::Instance(instance)
                }
            };

            items.push(item);
        }

        Ok((items, has_more_items))
    }

    #[expect(clippy::too_many_arguments)]
    async fn get_event_instances_as_stream<'a>(
        &'a self,
        current_user: &'a User,
        event_id: EventId,
        invitees_max: Option<PageSize>,
        time_min: Option<Timestamp>,
        time_min_inclusive: bool,
        time_max: Option<Timestamp>,
        instances_max: usize,
        offset: usize,
    ) -> Result<
        Pin<Box<dyn Stream<Item = opentalk_inventory::Result<EventInstance>> + 'a>>,
        CaptureApiError,
    > {
        let settings = self.settings_provider.get();

        let mut inventory = self.inventory_provider.get_inventory().await?;

        let (
            event,
            invite,
            room,
            sip_config,
            is_favorite,
            shared_folder,
            tariff,
            training_participation_report_parameter_set,
        ) = inventory
            .get_event_with_related_items(current_user.id, event_id)
            .await?;

        let (invitees, invitees_truncated) =
            super::get_invitees_for_event(&settings, inventory.as_mut(), event.id, invitees_max)
                .await?;

        let invite_status = invite
            .map(|inv| inv.status)
            .unwrap_or(EventInviteStatus::Accepted);

        let Some(rruleset) = event.to_rruleset()? else {
            return Err(ApiError::not_found().into());
        };

        const MONTHS_PER_YEAR: u32 = 12;

        // limit of how far into the future we calculate instances
        let max_dt = Utc::now()
            .with_timezone(&rruleset.get_dt_start().timezone())
            .checked_add_months(chrono::Months::new(40 * MONTHS_PER_YEAR))
            .expect("Could not add required duration");

        let mut datetimes: Box<dyn Iterator<Item = DateTime<rrule::Tz>>> =
            Box::new(rruleset.into_iter().take_while(move |&dt| dt <= max_dt));

        if let Some(datetime_min) = time_min {
            datetimes = Box::new(datetimes.skip_while(move |&dt| {
                dt < *datetime_min || !time_min_inclusive && dt == *datetime_min
            }));
        }

        if let Some(datetime_max) = time_max {
            datetimes = Box::new(datetimes.take_while(move |&dt| dt <= *datetime_max));
        }

        let datetimes: Vec<Timestamp> = datetimes
            .skip(offset)
            .take(instances_max)
            .map(|dt| dt.with_timezone(&Utc).into())
            .collect();

        let exceptions = inventory.get_event_exceptions(event_id, &datetimes).await?;

        let users = GetUserProfilesBatched::new()
            .add(&event)
            .add(&exceptions)
            .fetch(&settings, inventory.as_mut())
            .await?;

        let training_participation_report = training_participation_report_parameter_set
            .map(TrainingParticipationReportParameterSet::from);

        drop(inventory);

        let tariff = self.build_tariff_resource(&tariff)?;

        let room = EventRoomInfo::from_room(&settings, &room, sip_config.as_ref(), &tariff);

        let can_edit = current_user.can_edit(&event);

        let shared_folder =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

        let mut exceptions = exceptions.into_iter().peekable();

        let stream = stream! {
            for datetime in datetimes {
                let exception = exceptions.next_if(|exception| exception.exception_date == datetime);

                let event_instance = create_event_instance(
                    &users,
                    event.clone(),
                    invite_status,
                    is_favorite,
                    exception,
                    room.clone(),
                    datetime.into(),
                    invitees.clone(),
                    invitees_truncated,
                    can_edit,
                    shared_folder.clone(),
                    training_participation_report.clone(),
                );

                yield Ok(event_instance);
            }
        };

        Ok(Box::pin(stream))
    }

    pub(crate) async fn get_event_instances(
        &self,
        current_user: &RequestUser,
        event_id: EventId,
        GetEventInstancesQuery {
            invitees_max,
            time_min,
            time_max,
            per_page,
            after,
        }: GetEventInstancesQuery,
    ) -> Result<
        (
            GetEventInstancesResponseBody,
            Option<String>,
            Option<String>,
        ),
        CaptureApiError,
    > {
        let settings = self.settings_provider.get();

        let per_page = per_page.unwrap_or_default();
        let page = after.map(|c| c.page).unwrap_or_default();

        let items_per_page = per_page.into();
        let page_index = page.as_zero_based_usize();
        let offset = items_per_page * page_index;

        let mut inventory = self.inventory_provider.get_inventory().await?;

        let current_user = inventory.get_user(current_user.id).await?;

        let mut stream = self
            .get_event_instances_as_stream(
                &current_user,
                event_id,
                invitees_max,
                time_min,
                true,
                time_max,
                items_per_page,
                offset,
            )
            .await?;

        // Enrich the invitees for the first instance only and reuse them as all instances have the same invitees.
        let mut instances: Vec<EventInstance> = Vec::with_capacity(items_per_page);
        let mut reused_enriched_invitees: Option<Vec<EventInvitee>> = None;
        while let Some(instance) = stream.next().await {
            let instance = instance?;

            let enriched_invitees =
                if let Some(enriched_invitees) = reused_enriched_invitees.clone() {
                    enriched_invitees
                } else {
                    let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
                    let enriched_invitees = enrich_invitees_from_optional_user_search(
                        &settings,
                        &self.user_search_client,
                        &current_tenant,
                        instance.invitees.clone(),
                    )
                    .await;

                    reused_enriched_invitees = Some(enriched_invitees.clone());
                    enriched_invitees
                };

            let instance = EventInstance {
                invitees: enriched_invitees,
                ..instance
            };

            instances.push(instance)
        }

        let before = None;
        let after = if !instances.is_empty() {
            Some(
                Cursor(GetEventInstancesCursorData {
                    page: page.saturating_next(),
                })
                .to_base64(),
            )
        } else {
            None
        };

        Ok((GetEventInstancesResponseBody(instances), before, after))
    }

    pub(crate) async fn get_event_instance(
        &self,
        current_user: &RequestUser,
        EventInstancePath {
            event_id,
            instance_id,
        }: EventInstancePath,
        query: EventInstanceQuery,
    ) -> Result<GetEventInstanceResponseBody, CaptureApiError> {
        let mut inventory = self.inventory_provider.get_inventory().await?;

        let settings = self.settings_provider.get();

        let (
            event,
            invite,
            room,
            sip_config,
            is_favorite,
            shared_folder,
            tariff,
            training_participation_report_parameter_set,
        ) = inventory
            .get_event_with_related_items(current_user.id, event_id)
            .await?;

        let Some(event_date) = event.date() else {
            return Err(ApiError::not_found().into());
        };

        let instance_date = DateTime::from(instance_id).with_timezone(&event_date.starts_at_tz);

        verify_instance_date(event_date, instance_date)?;

        let (invitees, invitees_truncated) = super::get_invitees_for_event(
            &settings,
            inventory.as_mut(),
            event_id,
            query.invitees_max,
        )
        .await?;

        let exception = inventory
            .get_event_exception(event_id, instance_id.into())
            .await?;

        let users = GetUserProfilesBatched::new()
            .add(&event)
            .add(&exception)
            .fetch(&settings, inventory.as_mut())
            .await?;

        let tariff = self.build_tariff_resource(&tariff)?;

        let room = EventRoomInfo::from_room(&settings, &room, sip_config.as_ref(), &tariff);

        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
        let current_user = inventory.get_user(current_user.id).await?;

        let can_edit = current_user.can_edit(&event);

        let shared_folder =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

        let event_instance = create_event_instance(
            &users,
            event,
            invite
                .map(|inv| inv.status)
                .unwrap_or(EventInviteStatus::Accepted),
            is_favorite,
            exception,
            room,
            instance_id,
            invitees,
            invitees_truncated,
            can_edit,
            shared_folder,
            training_participation_report_parameter_set.map(Into::into),
        );

        let event_instance = EventInstance {
            invitees: enrich_invitees_from_optional_user_search(
                &settings,
                &self.user_search_client,
                &current_tenant,
                event_instance.invitees,
            )
            .await,
            ..event_instance
        };

        Ok(GetEventInstanceResponseBody(event_instance))
    }

    pub(crate) async fn patch_event_instance(
        &self,
        current_user: RequestUser,
        EventInstancePath {
            event_id,
            instance_id,
        }: EventInstancePath,
        EventInstanceQuery {
            invitees_max,
            suppress_email_notification,
        }: EventInstanceQuery,
        patch: PatchEventInstanceBody,
    ) -> Result<Option<EventInstance>, CaptureApiError> {
        if patch.is_empty() {
            return Ok(None);
        }

        let mut inventory = self.inventory_provider.get_inventory().await?;

        let settings = self.settings_provider.get();
        let default_user_language = Language(settings.defaults.user_language.clone());

        let (
            event,
            invite,
            room,
            sip_config,
            is_favorite,
            shared_folder,
            tariff,
            training_participation_report_parameter_set,
        ) = inventory
            .get_event_with_related_items(current_user.id, event_id)
            .await?;

        let Some(event_date) = event.date() else {
            return Err(ApiError::not_found().into());
        };

        let instance_date = DateTime::from(instance_id).with_timezone(&event_date.starts_at_tz);

        verify_instance_date(event_date, instance_date)?;

        let is_all_day = patch.is_all_day.unwrap_or(event_date.is_all_day);

        let starts_at = patch.starts_at.unwrap_or(DateTimeTz {
            datetime: event_date.starts_at.into(),
            timezone: event_date.starts_at_tz,
        });

        let ends_at = patch.ends_at.unwrap_or_else(|| {
            let (ends_at, ends_at_tz) = event_date.ends_at_of_first_occurrence();
            DateTimeTz {
                datetime: ends_at.into(),
                timezone: ends_at_tz,
            }
        });

        super::verify_exception_dt_params(is_all_day, starts_at, ends_at)?;

        let exception = match inventory
            .get_event_exception(event_id, instance_id.into())
            .await?
        {
            Some(exception) => {
                inventory
                    .update_event_exception(
                        exception.id,
                        UpdateEventException {
                            kind: match patch.status {
                                Some(EventStatus::Ok) => Some(EventExceptionKind::Modified),
                                Some(EventStatus::Cancelled) => Some(EventExceptionKind::Cancelled),
                                None => None,
                            },
                            title: patch.title.map(Some),
                            description: patch.description.map(Some),
                            is_all_day: patch.is_all_day.map(Some),
                            starts_at: patch.starts_at.map(|dt| Some(dt.to_datetime_tz())),
                            starts_at_tz: patch.starts_at.map(|dt| Some(dt.timezone)),
                            ends_at: patch.ends_at.map(|dt| Some(dt.to_datetime_tz())),
                            ends_at_tz: patch.ends_at.map(|dt| Some(dt.timezone)),
                        },
                    )
                    .await?
            }
            None => {
                inventory
                    .create_event_exception(NewEventException {
                        event_id: event.id,
                        exception_date: instance_id.into(),
                        exception_date_tz: event_date.starts_at_tz,
                        created_by: current_user.id,
                        kind: match patch.status {
                            Some(EventStatus::Cancelled) => EventExceptionKind::Cancelled,
                            _ => EventExceptionKind::Modified,
                        },
                        title: patch.title,
                        description: patch.description,
                        is_all_day: patch.is_all_day,
                        starts_at: patch.starts_at.map(|dt| dt.to_datetime_tz()),
                        starts_at_tz: patch.starts_at.map(|dt| dt.timezone),
                        ends_at: patch.ends_at.map(|dt| dt.to_datetime_tz()),
                        ends_at_tz: patch.ends_at.map(|dt| dt.timezone),
                    })
                    .await?
            }
        };

        let (invitees, invitees_truncated) =
            super::get_invitees_for_event(&settings, inventory.as_mut(), event_id, invitees_max)
                .await?;

        let users = GetUserProfilesBatched::new()
            .add(&event)
            .add(&exception)
            .fetch(&settings, inventory.as_mut())
            .await?;

        let tariff = self.build_tariff_resource(&tariff)?;

        let event_room_info =
            EventRoomInfo::from_room(&settings, &room, sip_config.as_ref(), &tariff);

        let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;
        let current_user = inventory.get_user(current_user.id).await?;

        let can_edit = current_user.can_edit(&event);

        let shared_folder =
            shared_folder_for_user(shared_folder, event.created_by, current_user.id);

        let streaming_targets = inventory.get_room_streaming_targets(room.id).await?;

        if !suppress_email_notification {
            let invited_users = get_invited_mail_recipients_for_event(
                inventory.as_mut(),
                event_id,
                default_user_language,
            )
            .await?;

            let created_by = if event.created_by == current_user.id {
                current_user
            } else {
                inventory.get_user(event.created_by).await?
            };

            if let Some(mail_service) = self.mail_service.as_ref() {
                let notification_values = UpdateNotificationValues {
                    tenant: current_tenant.clone(),
                    created_by,
                    event: event.clone(),
                    event_exception: Some(exception.clone()),
                    room,
                    sip_config,
                    users_to_notify: invited_users,
                };

                notify_invitees_about_update(
                    &settings,
                    &tariff,
                    notification_values,
                    mail_service,
                    &self.user_search_client,
                    None,
                    streaming_targets,
                )
                .await;
            }
        }

        drop(inventory);

        let event_instance = create_event_instance(
            &users,
            event,
            invite
                .map(|inv| inv.status)
                .unwrap_or(EventInviteStatus::Accepted),
            is_favorite,
            Some(exception),
            event_room_info,
            instance_id,
            invitees,
            invitees_truncated,
            can_edit,
            shared_folder,
            training_participation_report_parameter_set.map(Into::into),
        );

        let event_instance = EventInstance {
            invitees: enrich_invitees_from_optional_user_search(
                &settings,
                &self.user_search_client,
                &current_tenant,
                event_instance.invitees,
            )
            .await,
            ..event_instance
        };

        Ok(Some(event_instance))
    }
}

#[expect(clippy::too_many_arguments)]
fn create_event_instance(
    users: &UserProfilesBatch,
    mut event: Event,
    invite_status: EventInviteStatus,
    is_favorite: bool,
    exception: Option<EventException>,
    room: EventRoomInfo,
    instance_id: InstanceId,
    invitees: Vec<EventInvitee>,
    invitees_truncated: bool,
    can_edit: bool,
    shared_folder: Option<SharedFolder>,
    training_participation_report: Option<TrainingParticipationReportParameterSet>,
) -> EventInstance {
    let instance_date = event
        .date()
        .expect("event instances can only be created for events with a date");

    let instance_duration_secs = event
        .duration_secs()
        .expect("event instances can only be created for recurring events");

    let mut instance_starts_at = instance_id.into();
    let mut instance_starts_at_tz = instance_date.starts_at_tz;
    let mut instance_ends_at = instance_id + Duration::seconds(instance_duration_secs as i64);
    let mut instance_ends_at_tz = instance_date.ends_at_tz;

    let is_all_day = instance_date.is_all_day;

    let mut status = EventStatus::Ok;

    if let Some(exception) = exception {
        match exception.kind {
            EventExceptionKind::Modified => {} // Do nothing for now
            EventExceptionKind::Cancelled => status = EventStatus::Cancelled,
        }

        event.updated_by = exception.created_by;
        event.updated_at = exception.created_at;

        patch(&mut event.title, exception.title);
        patch(&mut event.description, exception.description);
        patch(&mut instance_starts_at, exception.starts_at);
        patch(&mut instance_starts_at_tz, exception.starts_at_tz);
        patch(
            &mut instance_ends_at,
            exception.ends_at.map(InstanceId::from),
        );
        patch(&mut instance_ends_at_tz, exception.ends_at_tz);
    }

    let created_by = users.get(event.created_by);
    let updated_by = users.get(event.updated_by);

    EventInstance {
        id: EventAndInstanceId(event.id, instance_id),
        recurring_event_id: event.id,
        instance_id,
        created_by,
        created_at: event.created_at,
        updated_by,
        updated_at: event.updated_at,
        title: event.title,
        description: event.description,
        room,
        invitees_truncated,
        invitees,
        is_all_day,
        starts_at: DateTimeTz {
            datetime: instance_starts_at.into(),
            timezone: instance_starts_at_tz,
        },
        ends_at: DateTimeTz {
            datetime: instance_ends_at.into(),
            timezone: instance_ends_at_tz,
        },
        type_: InstanceMarker::Instance,
        status,
        invite_status,
        is_favorite,
        can_edit,
        shared_folder,
        training_participation_report,
    }
}

fn patch<T>(dst: &mut T, value: Option<T>) {
    if let Some(value) = value {
        *dst = value;
    }
}

/// Consume up to `limit` items from `stream`, also reporting whether the stream
/// had more items beyond that limit. The (limit+1)th item, if any, is consumed
/// but not returned.
async fn take_with_has_more<T, E, S>(stream: S, limit: usize) -> Result<(Vec<T>, bool), E>
where
    S: Stream<Item = Result<T, E>>,
{
    pin_mut!(stream);
    let mut items = Vec::with_capacity(limit);
    for _ in 0..limit {
        let Some(result) = stream.next().await else {
            return Ok((items, false));
        };
        items.push(result?);
    }
    let has_more = stream.next().await.transpose()?.is_some();
    Ok((items, has_more))
}

/// Build the `after` pagination cursor from a page of results.
///
/// Returns `None` when `has_more_items` is false (i.e. this is the last page),
/// even if `items` is non-empty. This prevents clients from being handed a
/// cursor that would yield an empty next page.
fn build_after_cursor(items: &[EventOrInstance], has_more_items: bool) -> Option<String> {
    if !has_more_items {
        return None;
    }
    items.last().map(|item| {
        let (event_id, event_created_at, event_starts_at, instance_id) = match item {
            EventOrInstance::Event(event) => (
                event.id,
                event.created_at,
                event.date.starts_at().cloned(),
                None,
            ),
            EventOrInstance::Instance(instance) => (
                instance.recurring_event_id,
                instance.created_at,
                Some(instance.starts_at),
                Some(instance.instance_id),
            ),
        };

        Cursor(GetEventsCursorData {
            event_id,
            event_created_at,
            event_starts_at: event_starts_at.map(Into::into),
            instance_id,
        })
        .to_base64()
    })
}

fn verify_instance_date(
    event_date: &EventDate,
    instance_date: DateTime<TimeZone>,
) -> Result<(), ApiError> {
    if instance_date > DateTime::from(event_date.starts_at) + ONE_HUNDRED_YEARS {
        // TODO(t.spamer): I would argue this is a
        // ApiError::bad_request().with_message("requested date exceeds maximum 100-year range").
        // Change in follow up MR.
        return Err(ApiError::not_found());
    }

    let Some(rruleset) = event_date.to_rruleset()? else {
        return Err(ApiError::not_found());
    };

    let has_instance = rruleset
        .into_iter()
        .take_while(|dt| dt <= &instance_date)
        .any(|x| x == instance_date);

    if !has_instance {
        return Err(ApiError::not_found());
    };

    Ok(())
}

fn compare_inventory_events_or_instances(
    item_1: &InternalEventOrInstance,
    item_2: &InternalEventOrInstance,
) -> Ordering {
    let (starts_at_1, created_at_1, event_id_1, instance_id_1) = match item_1 {
        InternalEventOrInstance::Event((
            event,
            _invite,
            _room,
            _sip_config,
            _is_favorite,
            _shared_folder,
            _tariff,
            _training_participation_report_parameter_set,
        )) => (
            event.date().map(|date| date.starts_at),
            event.created_at,
            event.id,
            None,
        ),
        InternalEventOrInstance::Instance(event_instance) => (
            Some(event_instance.starts_at.into()),
            event_instance.created_at,
            event_instance.id.0,
            Some(event_instance.instance_id),
        ),
    };

    let (starts_at_2, created_at_2, event_id_2, instance_id_2) = match item_2 {
        InternalEventOrInstance::Event((
            event,
            _invite,
            _room,
            _sip_config,
            _is_favorite,
            _shared_folder,
            _tariff,
            _training_participation_report_parameter_set,
        )) => (
            event.date().map(|date| date.starts_at),
            event.created_at,
            event.id,
            None,
        ),
        InternalEventOrInstance::Instance(event_instance) => (
            Some(event_instance.starts_at.into()),
            event_instance.created_at,
            event_instance.id.0,
            Some(event_instance.instance_id),
        ),
    };

    // This comparison requires that the streams are properly sorted with NULLs (i.e. `None` values) first.
    match (starts_at_1, starts_at_2) {
        (None, _) => Ordering::Less,
        (_, None) => Ordering::Greater,
        (Some(starts_at_1), Some(starts_at_2)) => starts_at_1.cmp(&starts_at_2),
    }
    .then(created_at_1.cmp(&created_at_2))
    .then(event_id_1.cmp(&event_id_2))
    .then(match (instance_id_1, instance_id_2) {
        (None, _) => Ordering::Less,
        (_, None) => Ordering::Greater,
        (Some(instance_id_1), Some(instance_id_2)) => instance_id_1.cmp(&instance_id_2),
    })
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use chrono_tz::Tz;
    use opentalk_types_api_v1::{
        events::{EventInviteeProfile, PublicInviteUserProfile},
        users::PublicUserProfile,
    };
    use opentalk_types_common::{
        events::{EventId, invites::InviteRole},
        rooms::{GuestAccess, RoomId},
        time::{TimeZone, Timestamp},
        users::{UserId, UserInfo},
    };
    use serde_json::json;

    use super::*;

    #[test]
    fn event_instance_serialize() {
        let unix_epoch: Timestamp = SystemTime::UNIX_EPOCH.into();
        let instance_id = unix_epoch.into();
        let event_id = EventId::nil();
        let user_profile = PublicUserProfile {
            id: UserId::nil(),
            email: "test@example.org".into(),
            user_info: UserInfo {
                title: "".parse().expect("valid user title"),
                firstname: "Test".into(),
                lastname: "Test".into(),
                display_name: "Tester".parse().expect("valid display name"),
                avatar_url: "https://example.org/avatar".into(),
            },
        };

        let instance = EventInstance {
            id: EventAndInstanceId(event_id, instance_id),
            recurring_event_id: event_id,
            instance_id,
            created_by: user_profile.clone(),
            created_at: unix_epoch,
            updated_by: user_profile.clone(),
            updated_at: unix_epoch,
            title: "Instance title".parse().expect("valid event title"),
            description: "Instance description"
                .parse()
                .expect("valid event description"),
            room: EventRoomInfo {
                id: RoomId::nil(),
                alias: None,
                password: None,
                waiting_room: false,
                guest_access: GuestAccess::default(),
                e2e_encryption: false,
                call_in: None,
            },
            invitees_truncated: false,
            invitees: vec![EventInvitee {
                profile: EventInviteeProfile::Registered(PublicInviteUserProfile {
                    user_profile,
                    role: InviteRole::User,
                }),
                status: EventInviteStatus::Accepted,
            }],
            is_all_day: false,
            starts_at: DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            },
            ends_at: DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            },
            type_: InstanceMarker::Instance,
            status: EventStatus::Ok,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: None,
            training_participation_report: None,
        };

        assert_eq!(
            json!(instance),
            json!({
                    "id": "00000000-0000-0000-0000-000000000000_19700101T000000Z",
                    "recurring_event_id": "00000000-0000-0000-0000-000000000000",
                    "instance_id": "19700101T000000Z",
                    "created_by": {
                        "id": "00000000-0000-0000-0000-000000000000",
                        "email": "test@example.org",
                        "title": "",
                        "firstname": "Test",
                        "lastname": "Test",
                        "display_name": "Tester",
                        "avatar_url": "https://example.org/avatar"
                    },
                    "created_at": "1970-01-01T00:00:00Z",
                    "updated_by": {
                        "id": "00000000-0000-0000-0000-000000000000",
                        "email": "test@example.org",
                        "title": "",
                        "firstname": "Test",
                        "lastname": "Test",
                        "display_name": "Tester",
                        "avatar_url": "https://example.org/avatar"
                    },
                    "updated_at": "1970-01-01T00:00:00Z",
                    "title": "Instance title",
                    "description": "Instance description",
                    "room": {
                        "id": "00000000-0000-0000-0000-000000000000",
                        "waiting_room": false,
                        "guest_access": "waiting_room",
                        "e2e_encryption": false
                    },
                    "invitees_truncated": false,
                    "invitees": [
                        {
                            "profile": {
                                "kind": "registered",
                                "id": "00000000-0000-0000-0000-000000000000",
                                "email": "test@example.org",
                                "title": "",
                                "firstname": "Test",
                                "lastname": "Test",
                                "display_name": "Tester",
                                "avatar_url": "https://example.org/avatar",
                                "role": "user"

                            },
                            "status": "accepted"
                        }
                    ],
                    "is_all_day": false,
                    "starts_at": {
                        "datetime": "1970-01-01T00:00:00Z",
                        "timezone": "Europe/Berlin"
                    },
                    "ends_at": {
                        "datetime": "1970-01-01T00:00:00Z",
                        "timezone": "Europe/Berlin"
                    },
                    "type": "instance",
                    "status": "ok",
                    "invite_status": "accepted",
                    "is_favorite": false,
                    "can_edit": false,
                }
            )
        );
    }

    fn sample_event_or_instance() -> EventOrInstance {
        let unix_epoch: Timestamp = SystemTime::UNIX_EPOCH.into();
        let instance_id = unix_epoch.into();
        let event_id = EventId::nil();
        let user_profile = PublicUserProfile {
            id: UserId::nil(),
            email: "test@example.org".into(),
            user_info: UserInfo {
                title: "".parse().expect("valid user title"),
                firstname: "Test".into(),
                lastname: "Test".into(),
                display_name: "Tester".parse().expect("valid display name"),
                avatar_url: "https://example.org/avatar".into(),
            },
        };

        EventOrInstance::Instance(EventInstance {
            id: EventAndInstanceId(event_id, instance_id),
            recurring_event_id: event_id,
            instance_id,
            created_by: user_profile.clone(),
            created_at: unix_epoch,
            updated_by: user_profile,
            updated_at: unix_epoch,
            title: "Instance title".parse().expect("valid event title"),
            description: "Instance description"
                .parse()
                .expect("valid event description"),
            room: EventRoomInfo {
                id: RoomId::nil(),
                alias: None,
                password: None,
                waiting_room: false,
                guest_access: GuestAccess::default(),
                e2e_encryption: false,
                call_in: None,
            },
            invitees_truncated: false,
            invitees: vec![],
            is_all_day: false,
            starts_at: DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            },
            ends_at: DateTimeTz {
                datetime: *unix_epoch,
                timezone: TimeZone::from(Tz::Europe__Berlin),
            },
            type_: InstanceMarker::Instance,
            status: EventStatus::Ok,
            invite_status: EventInviteStatus::Accepted,
            is_favorite: false,
            can_edit: false,
            shared_folder: None,
            training_participation_report: None,
        })
    }

    #[test]
    fn build_after_cursor_returns_none_when_no_more_items() {
        let items = vec![sample_event_or_instance()];
        assert_eq!(build_after_cursor(&items, false), None);
    }

    #[test]
    fn build_after_cursor_returns_none_when_items_empty() {
        assert_eq!(build_after_cursor(&[], true), None);
    }

    #[test]
    fn build_after_cursor_returns_some_when_more_items_exist() {
        let items = vec![sample_event_or_instance()];
        let cursor = build_after_cursor(&items, true);
        assert!(
            cursor.is_some(),
            "expected a cursor when has_more_items is true"
        );
        assert!(!cursor.unwrap().is_empty());
    }

    #[test]
    fn take_with_has_more_empty_stream() {
        let stream = futures_util::stream::iter(Vec::<Result<i32, &'static str>>::new());
        let (items, has_more) =
            futures::executor::block_on(take_with_has_more(stream, 5)).expect("ok");
        assert!(items.is_empty());
        assert!(!has_more);
    }

    #[test]
    fn take_with_has_more_fewer_than_limit() {
        let stream = futures_util::stream::iter(vec![Ok::<_, &'static str>(1), Ok(2)]);
        let (items, has_more) =
            futures::executor::block_on(take_with_has_more(stream, 5)).expect("ok");
        assert_eq!(items, vec![1, 2]);
        assert!(!has_more);
    }

    #[test]
    fn take_with_has_more_exactly_limit() {
        let stream = futures_util::stream::iter(vec![Ok::<_, &'static str>(1), Ok(2), Ok(3)]);
        let (items, has_more) =
            futures::executor::block_on(take_with_has_more(stream, 3)).expect("ok");
        assert_eq!(items, vec![1, 2, 3]);
        assert!(
            !has_more,
            "no cursor must be emitted when exactly `limit` items exist"
        );
    }

    #[test]
    fn take_with_has_more_more_than_limit() {
        let stream =
            futures_util::stream::iter(vec![Ok::<_, &'static str>(1), Ok(2), Ok(3), Ok(4)]);
        let (items, has_more) =
            futures::executor::block_on(take_with_has_more(stream, 3)).expect("ok");
        assert_eq!(items, vec![1, 2, 3]);
        assert!(has_more);
    }

    #[test]
    fn take_with_has_more_zero_limit() {
        let stream = futures_util::stream::iter(vec![Ok::<_, &'static str>(1)]);
        let (items, has_more) =
            futures::executor::block_on(take_with_has_more(stream, 0)).expect("ok");
        assert!(items.is_empty());
        assert!(has_more);
    }

    #[test]
    fn take_with_has_more_propagates_error_within_limit() {
        let stream = futures_util::stream::iter(vec![Ok(1), Err::<i32, _>("boom"), Ok(3)]);
        let err = futures::executor::block_on(take_with_has_more(stream, 5));
        assert_eq!(err, Err("boom"));
    }

    #[test]
    fn take_with_has_more_propagates_error_at_peek() {
        let stream = futures_util::stream::iter(vec![Ok(1), Ok(2), Err::<i32, _>("peek-err")]);
        let err = futures::executor::block_on(take_with_has_more(stream, 2));
        assert_eq!(err, Err("peek-err"));
    }
}
