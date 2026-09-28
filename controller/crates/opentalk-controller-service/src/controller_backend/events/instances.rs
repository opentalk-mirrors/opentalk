// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Handles event instances

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
    pin::Pin,
    sync::Arc,
};

use async_stream::{stream, try_stream};
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
        GetEventsAndInstancesQuery, InstanceId, InstanceMarker, PatchEventInstanceBody,
    },
    pagination::Cursor,
};
use opentalk_types_common::{
    events::{EventId, invites::EventInviteStatus},
    order::Ordering as SortOrdering,
    pagination::PageSize,
    shared_folders::SharedFolder,
    time::{DateTimeTz, TimeZone, Timestamp},
    training_participation_report::TrainingParticipationReportParameterSet,
    users::Language,
};

use crate::{
    ControllerBackend,
    controller_backend::{
        events::{
            DateTimeTzFromInventory, EventRoomInfoExt, ONE_HUNDRED_YEARS_IN_DAYS,
            pagination::EventsPage,
        },
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

        // A "past meetings" query (only `time_max`, no `time_min`) is ordered
        // closest-to-`time_max` first, so the rest of this function must merge in that order too.
        let descending = query.time_min.is_none() && query.time_max.is_some();
        let ordering = if descending {
            SortOrdering::Descending
        } else {
            SortOrdering::Ascending
        };

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
                ordering,
            )
            .await?;

        // Return all non-recurring events, but recurring events only if event instances are
        // suppressed
        let events_stream = events_stream.filter(|result| {
            futures::future::ready({
                if let Ok(stream_item) = result {
                    !stream_item.0.is_recurring() || query.instances_max.is_zero()
                } else {
                    true
                }
            })
        });

        // Prepare all streams to interweave (one for the events and one for each event's
        // instances).
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
        let cursor_event_starts_at = query.after.and_then(|cursor| cursor.event_starts_at);
        let query_time_min = query.time_min;
        let query_time_max = query.time_max;

        // The cursor bound is exclusive so the next page starts after the last returned instance.
        // Descending mode has no `time_min`, so the cursor narrows `time_max` instead.
        let (time_min, time_min_inclusive) = if descending {
            (None, true)
        } else if let Some(from_cursor) = cursor_event_starts_at
            && query_time_min.is_none_or(|time_min| from_cursor >= time_min)
        {
            (Some(from_cursor), false)
        } else {
            (query_time_min, true)
        };

        let (time_max, time_max_inclusive) = if descending
            && let Some(from_cursor) = cursor_event_starts_at
            && query_time_max.is_none_or(|time_max| from_cursor <= time_max)
        {
            (Some(from_cursor), false)
        } else {
            (query_time_max, true)
        };

        for event in recurring_events {
            let instances_stream = self
                .get_event_instances_as_stream(
                    &current_user,
                    event.id,
                    query.invitees_max,
                    time_min,
                    time_min_inclusive,
                    time_max,
                    time_max_inclusive,
                    query.instances_max.into(),
                    0,
                    descending,
                )
                .await?;

            events_and_instances_streams.push(Box::pin(
                instances_stream.map_ok(InternalEventOrInstance::Instance),
            ));
        }

        // Interweave the streams, using a comparator.
        let comparator = if descending {
            compare_inventory_events_or_instances_desc
        } else {
            compare_inventory_events_or_instances
        };
        let stream = interweave_result_streams(events_and_instances_streams, comparator);

        let settings = self.settings_provider.get();
        let page = self
            .create_events_or_instances_page_from_stream(
                self.inventory_provider.clone(),
                &settings,
                current_user.clone(),
                stream,
                query.invitees_max,
                query.per_page,
                ordering,
            )
            .await?;

        Ok(page.into_parts())
    }

    #[expect(clippy::too_many_arguments)]
    async fn create_events_or_instances_page_from_stream(
        &self,
        inventory_provider: Arc<dyn InventoryProvider>,
        settings: &Settings,
        current_user: User,
        stream: impl Stream<Item = opentalk_inventory::Result<InternalEventOrInstance>>,
        invitees_max: Option<PageSize>,
        per_page: Option<PageSize>,
        ordering: SortOrdering,
    ) -> Result<EventsPage<EventOrInstance>, CaptureApiError> {
        let resources = try_stream! {
            let mut inventory = inventory_provider.get_inventory().await?;
            let current_tenant = inventory.get_tenant(current_user.tenant_id).await?;

            pin_mut!(stream);
            while let Some(result) = stream.next().await {
                let item = match result? {
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

                yield item;
            }
        };

        EventsPage::from_instances(resources, per_page, ordering).await
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
        time_max_inclusive: bool,
        instances_max: usize,
        offset: usize,
        descending: bool,
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
            datetimes = Box::new(datetimes.take_while(move |&dt| {
                dt < *datetime_max || time_max_inclusive && dt == *datetime_max
            }));
        }

        let datetimes: Vec<Timestamp> = if descending {
            // RRULE occurrences can only be walked forward, so keep a trailing window
            // of the last `instances_max` occurrences to get the ones closest to `time_max`.
            let mut window: VecDeque<DateTime<rrule::Tz>> = VecDeque::with_capacity(instances_max);
            for dt in datetimes {
                if instances_max == 0 {
                    break;
                }
                if window.len() == instances_max {
                    window.pop_front();
                }
                window.push_back(dt);
            }
            window
                .into_iter()
                .rev()
                .map(|dt| dt.with_timezone(&Utc).into())
                .collect()
        } else {
            datetimes
                .skip(offset)
                .take(instances_max)
                .map(|dt| dt.with_timezone(&Utc).into())
                .collect()
        };

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

        // A map, not position-based matching, since `datetimes` may not be ascending anymore.
        let mut exceptions: BTreeMap<Timestamp, EventException> = exceptions
            .into_iter()
            .map(|exception| (exception.exception_date, exception))
            .collect();

        let stream = stream! {
            for datetime in datetimes {
                let exception = exceptions.remove(&datetime);

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
                true,
                items_per_page,
                offset,
                false,
            )
            .await?;

        // Enrich the invitees for the first instance only and reuse them as all instances have the
        // same invitees.
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

    // This comparison requires that the streams are properly sorted with NULLs (i.e. `None` values)
    // first.
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

/// The descending counterpart of [`compare_inventory_events_or_instances`], used to
/// merge streams that are each sorted closest-to-`time_max`-first.
fn compare_inventory_events_or_instances_desc(
    item_1: &InternalEventOrInstance,
    item_2: &InternalEventOrInstance,
) -> Ordering {
    compare_inventory_events_or_instances(item_1, item_2).reverse()
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

    /// Regression coverage for the "GET /events/instances" endpoint: a meeting that
    /// has just ended must show up in the "Past Meetings" list, the same way it
    /// correctly showed up in the "future" list before it started and correctly
    /// disappeared from it once it ended.
    #[cfg(feature = "mockall")]
    mod past_meetings_regression {
        use std::{collections::BTreeMap, sync::Arc};

        use futures_util::stream;
        use opentalk_controller_api_authorization::authorization::{
            Authorizer, MockAuthorizerBackend,
        };
        use opentalk_controller_settings::{MockSettingsProvider, Settings, test_util};
        use opentalk_inventory::{
            EventRecurrence, Inventory, MockInventory, MockInventoryProvider, Tenant,
        };
        use opentalk_types_api_v1::events::GetEventsCursorData;
        use opentalk_types_common::{tariffs::TariffId, tenants::TenantId, utils::ExampleData};

        use super::*;
        use crate::{
            ControllerBackend,
            controller_backend::roomserver::MockRoomServerBackend,
            oidc::{Cache, MockOidcTokenHandler},
        };

        fn request_user(user: &User) -> RequestUser {
            RequestUser {
                id: user.id,
                email: user.email.clone(),
                title: user.title.clone(),
                firstname: user.firstname.clone(),
                lastname: user.lastname.clone(),
                language: user.language.clone(),
                display_name: user.display_name.clone(),
                dashboard_theme: user.dashboard_theme,
                conference_theme: user.conference_theme,
                tenant_id: user.tenant_id,
                tariff_id: user.tariff_id,
                tariff_status: user.tariff_status,
                avatar_url: user.avatar_url.clone(),
            }
        }

        fn test_room(id: RoomId, created_by: UserId, tenant_id: TenantId) -> Room {
            Room {
                id,
                alias: None,
                id_serial: 0,
                created_by,
                created_at: Timestamp::unix_epoch(),
                password: None,
                waiting_room: false,
                guest_access: GuestAccess::default(),
                tenant_id,
                e2e_encryption: false,
            }
        }

        /// A single (non-recurring) scheduled meeting with the given start/end.
        fn scheduled_meeting(
            id: EventId,
            created_by: UserId,
            tenant_id: TenantId,
            starts_at: Timestamp,
            ends_at: Timestamp,
        ) -> Event {
            Event {
                id,
                id_serial: 0,
                title: "Team sync".parse().expect("valid event title"),
                description: "".parse().expect("valid event description"),
                room: RoomId::nil(),
                created_by,
                created_at: starts_at,
                updated_by: created_by,
                updated_at: starts_at,
                is_adhoc: false,
                tenant_id,
                revision: 0,
                show_meeting_details: true,
                date: Some(EventDate {
                    is_all_day: false,
                    starts_at,
                    starts_at_tz: TimeZone::utc(),
                    ends_at,
                    ends_at_tz: TimeZone::utc(),
                    duration_secs: (*ends_at - *starts_at).num_seconds() as i32,
                    recurrence: None,
                }),
            }
        }

        /// A recurring meeting whose first occurrence starts at `starts_at`, repeating
        /// according to `recurrence_pattern` (an RRULE string, e.g. `"FREQ=DAILY"`).
        fn recurring_meeting(
            id: EventId,
            created_by: UserId,
            tenant_id: TenantId,
            starts_at: Timestamp,
            ends_at: Timestamp,
            recurrence_pattern: &str,
        ) -> Event {
            Event {
                id,
                id_serial: 0,
                title: "Recurring sync".parse().expect("valid event title"),
                description: "".parse().expect("valid event description"),
                room: RoomId::nil(),
                created_by,
                created_at: starts_at,
                updated_by: created_by,
                updated_at: starts_at,
                is_adhoc: false,
                tenant_id,
                revision: 0,
                show_meeting_details: true,
                date: Some(EventDate {
                    is_all_day: false,
                    starts_at,
                    starts_at_tz: TimeZone::utc(),
                    ends_at,
                    ends_at_tz: TimeZone::utc(),
                    duration_secs: (*ends_at - *starts_at).num_seconds() as i32,
                    recurrence: Some(EventRecurrence {
                        recurrence_pattern: recurrence_pattern.to_string(),
                    }),
                }),
            }
        }

        fn test_tariff() -> Tariff {
            Tariff {
                id: TariffId::nil(),
                name: "test".to_string(),
                created_at: Timestamp::unix_epoch(),
                updated_at: Timestamp::unix_epoch(),
                quotas: Default::default(),
                disabled_modules: Vec::new(),
                disabled_features: Vec::new(),
            }
        }

        type EventStreamItem = (
            Event,
            Option<EventInvite>,
            Room,
            Option<RoomSipConfig>,
            bool,
            Option<EventSharedFolder>,
            Tariff,
            Option<TrainingParticipationReportParameterSet>,
        );

        fn stream_item(event: Event, tenant_id: TenantId) -> EventStreamItem {
            let room = test_room(event.room, event.created_by, tenant_id);
            (event, None, room, None, false, None, test_tariff(), None)
        }

        /// Builds an [`InventoryProvider`] mock that, on every `get_inventory()` call,
        /// hands out a fresh [`MockInventory`] backing the handful of inventory calls
        /// this scenario needs.
        fn inventory_provider(
            user: User,
            tenant: Tenant,
            events_in_ascending_order: Vec<Event>,
            recurring_events: Vec<Event>,
        ) -> MockInventoryProvider {
            let mut provider = MockInventoryProvider::new();
            provider.expect_get_inventory().returning(move || {
                let mut inventory = MockInventory::new();

                let get_user = user.clone();
                inventory
                    .expect_get_user()
                    .returning(move |_| Ok(get_user.clone()));

                let get_users_by_ids = user.clone();
                inventory.expect_get_users_by_ids().returning(move |ids| {
                    Ok(ids.iter().map(|_| get_users_by_ids.clone()).collect())
                });

                let get_tenant = tenant.clone();
                inventory
                    .expect_get_tenant()
                    .returning(move |_| Ok(get_tenant.clone()));

                let all_events_for_user = recurring_events.clone();
                inventory
                    .expect_get_all_events_for_user()
                    .returning(move |_, _| Ok(all_events_for_user.clone()));

                let events_with_related_items = recurring_events.clone();
                let tenant_id = tenant.id;
                inventory
                    .expect_get_event_with_related_items()
                    .returning(move |_, event_id| {
                        let event = events_with_related_items
                            .iter()
                            .find(|event| event.id == event_id)
                            .cloned()
                            .expect("event must be one of the configured recurring events");
                        let room = test_room(event.room, event.created_by, tenant_id);
                        Ok((event, None, room, None, false, None, test_tariff(), None))
                    });

                inventory
                    .expect_get_event_exceptions()
                    .returning(|_, _| Ok(Vec::new()));

                inventory
                    .expect_get_all_event_exceptions_for_user_paginated_as_stream()
                    .returning(|_, _, _, _, _, _, _, _, _, _| Ok(Box::pin(stream::empty())));

                let items: Vec<EventStreamItem> = events_in_ascending_order
                    .iter()
                    .cloned()
                    .map(|event| stream_item(event, tenant.id))
                    .collect();
                inventory
                    .expect_get_all_events_for_user_paginated_as_stream()
                    .returning(move |_, _, _, _, _, _, _, _, _, _, ordering| {
                        let mut items = items.clone();
                        if ordering == SortOrdering::Descending {
                            items.reverse();
                        }
                        Ok(Box::pin(stream::iter(items.into_iter().map(Ok))) as _)
                    });

                Ok(Box::new(inventory) as Box<dyn Inventory>)
            });
            provider
        }

        fn build_backend(
            user: User,
            tenant: Tenant,
            events_in_ascending_order: Vec<Event>,
            recurring_events: Vec<Event>,
        ) -> ControllerBackend {
            let mut settings_provider = MockSettingsProvider::new();
            let settings: Arc<Settings> =
                test_util::settings_provider_from_example_raw_settings().get();
            settings_provider
                .expect_get()
                .returning(move || settings.clone());

            let provider =
                inventory_provider(user, tenant, events_in_ascending_order, recurring_events);

            ControllerBackend::new(
                settings_provider,
                Authorizer::new(MockAuthorizerBackend::new()),
                Arc::new(provider),
                Arc::new(Cache::create(None)),
                Arc::new(MockOidcTokenHandler::new()),
                opentalk_types_api_v1::auth::OidcProvider {
                    name: "test".to_string(),
                    url: "https://example.org".to_string(),
                },
                Arc::new(opentalk_asset_storage::ObjectStorage::broken()),
                Arc::new(None),
                Arc::new(None),
                BTreeMap::new(),
                Arc::new(MockRoomServerBackend::new()),
            )
        }

        #[tokio::test]
        async fn events_endpoint_keeps_past_events_in_ascending_order() {
            use opentalk_types_api_v1::events::{EventOrException, GetEventsQuery};

            let user = User::example_data();
            let tenant = Tenant::example_data();
            let now = Timestamp::now();
            let mut events = Vec::new();
            for day in 1..=3 {
                let starts_at = now + chrono::Duration::days(day - 4);
                events.push(scheduled_meeting(
                    EventId::from_u128(day as u128),
                    user.id,
                    tenant.id,
                    starts_at,
                    starts_at + chrono::Duration::hours(1),
                ));
            }

            let backend = build_backend(user.clone(), tenant, events, Vec::new());
            let query = GetEventsQuery {
                time_max: Some(now),
                per_page: Some(PageSize::from_i64_clamped(2)),
                ..Default::default()
            };

            let (items, _, after) = backend
                .get_events_and_exceptions_interwoven(request_user(&user), query)
                .await
                .expect("query must succeed");

            let ids: Vec<_> = items
                .iter()
                .map(|item| match item {
                    EventOrException::Event(event) => event.id,
                    EventOrException::Exception(_) => panic!("expected an event"),
                })
                .collect();
            assert_eq!(ids, vec![EventId::from_u128(1), EventId::from_u128(2)]);

            let cursor = Cursor::<GetEventsCursorData>::from_base64(&after.unwrap()).unwrap();
            assert_eq!(cursor.event_id, EventId::from_u128(2));
        }

        #[tokio::test]
        async fn meeting_appears_in_future_list_before_it_starts() {
            let user = User::example_data();
            let tenant = Tenant::example_data();
            let now = Timestamp::now();
            let meeting = scheduled_meeting(
                EventId::from_u128(1),
                user.id,
                tenant.id,
                now + chrono::Duration::hours(1),
                now + chrono::Duration::hours(2),
            );

            let backend = build_backend(
                user.clone(),
                tenant.clone(),
                vec![meeting.clone()],
                Vec::new(),
            );
            let query = GetEventsAndInstancesQuery {
                time_min: Some(now),
                ..Default::default()
            };

            let (items, _before, _after) = backend
                .get_events_and_instances_interwoven(request_user(&user), query)
                .await
                .expect("query must succeed");

            assert!(
                items
                    .iter()
                    .any(|item| matches!(item, EventOrInstance::Event(e) if e.id == meeting.id)),
                "the upcoming meeting must be visible in the future meetings list before it starts"
            );
        }

        #[tokio::test]
        async fn meeting_is_removed_from_future_list_once_it_has_ended() {
            let user = User::example_data();
            let tenant = Tenant::example_data();
            let now = Timestamp::now();
            let backend = build_backend(user.clone(), tenant.clone(), Vec::new(), Vec::new());

            let query = GetEventsAndInstancesQuery {
                time_min: Some(now),
                ..Default::default()
            };

            let (items, _before, _after) = backend
                .get_events_and_instances_interwoven(request_user(&user), query)
                .await
                .expect("query must succeed");

            assert!(
                items.is_empty(),
                "a meeting that has already ended must no longer be shown in the future meetings list"
            );
        }

        #[tokio::test]
        async fn recently_ended_meeting_included_in_the_past_meetings_list() {
            const EVENTS_PER_PAGE: i64 = 10;

            let user = User::example_data();
            let tenant = Tenant::example_data();
            let now = Timestamp::now();

            // Fill entire page with events.
            let mut events: Vec<Event> = (0..EVENTS_PER_PAGE)
                .map(|i| {
                    scheduled_meeting(
                        EventId::from_u128((100 + i) as u128),
                        user.id,
                        tenant.id,
                        now + chrono::Duration::days(-365) + chrono::Duration::hours(i),
                        now + chrono::Duration::days(-365) + chrono::Duration::hours(i + 1),
                    )
                })
                .collect();

            let recently_ended_meeting = scheduled_meeting(
                EventId::from_u128(999),
                user.id,
                tenant.id,
                now + chrono::Duration::hours(-2),
                now + chrono::Duration::hours(-1),
            );
            events.push(recently_ended_meeting.clone());

            let backend = build_backend(user.clone(), tenant.clone(), events, Vec::new());
            let page_size = PageSize::from_i64_clamped(EVENTS_PER_PAGE);

            let query = GetEventsAndInstancesQuery {
                time_max: Some(now),
                per_page: Some(page_size),
                ..Default::default()
            };

            let (items, _before, after) = backend
                .get_events_and_instances_interwoven(request_user(&user), query)
                .await
                .expect("query must succeed");

            assert!(
                items
                    .iter()
                    .any(|item| matches!(item, EventOrInstance::Event(e) if e.id == recently_ended_meeting.id)),
                "the meeting that just ended must be visible in the past meetings list"
            );

            let ids: Vec<_> = items
                .iter()
                .map(|item| match item {
                    EventOrInstance::Event(event) => event.id,
                    EventOrInstance::Instance(_) => panic!("expected an event"),
                })
                .collect();
            let mut expected_ids: Vec<_> = (101..110).map(EventId::from_u128).collect();
            expected_ids.push(recently_ended_meeting.id);
            assert_eq!(ids, expected_ids);

            let cursor = Cursor::<GetEventsCursorData>::from_base64(&after.unwrap()).unwrap();
            assert_eq!(cursor.event_id, EventId::from_u128(101));
        }

        /// A recurring meeting's oldest occurrence must not push its newest one out
        /// of the results.
        #[tokio::test]
        async fn recent_instance_of_long_running_recurring_meeting_is_not_excluded() {
            let user = User::example_data();
            let tenant = Tenant::example_data();
            let now = Timestamp::now();

            // Today's occurrence ended 2h ago; the first was 730 occurrences earlier.
            let just_ended_instance_starts_at = now + chrono::Duration::hours(-2);
            let anchor_starts_at = just_ended_instance_starts_at + chrono::Duration::days(-730);
            let anchor_ends_at = anchor_starts_at + chrono::Duration::hours(1);

            let long_running_meeting = recurring_meeting(
                EventId::from_u128(1),
                user.id,
                tenant.id,
                anchor_starts_at,
                anchor_ends_at,
                "FREQ=DAILY;INTERVAL=1",
            );

            let backend = build_backend(
                user.clone(),
                tenant.clone(),
                Vec::new(),
                vec![long_running_meeting.clone()],
            );

            let query = GetEventsAndInstancesQuery {
                time_max: Some(now),
                per_page: Some(PageSize::from_i64_clamped(10)),
                // Small enough that 730 older occurrences would fill the window first.
                instances_max: 3.into(),
                ..Default::default()
            };

            let (items, _before, _after) = backend
                .get_events_and_instances_interwoven(request_user(&user), query)
                .await
                .expect("query must succeed");

            assert!(
                items.iter().any(|item| matches!(
                    item,
                    EventOrInstance::Instance(instance)
                        if instance.recurring_event_id == long_running_meeting.id
                            && instance.starts_at.datetime > *(now + chrono::Duration::hours(-3))
                )),
                "the most recent instance of a long-running recurring meeting must be \
                 visible in the past meetings list, not just its oldest occurrences"
            );
        }
    }
}
