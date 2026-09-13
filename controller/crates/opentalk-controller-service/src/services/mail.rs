// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! MailService
//!
//! Used to have a clean interface for various kinds of mails
//! that are sent from the Web-API and possibly other connected services.
//!
// TODO: We probably can avoid the conversion to MailTasks if no rabbit_mq_queue is set in all mail fns
use std::sync::Arc;

use chrono::DateTime;
use lapin::BasicProperties;
use lapin_pool::{RabbitMqChannel, RabbitMqPool};
use opentalk_controller_settings::Settings;
use opentalk_inventory::{
    Event, EventException, EventExceptionKind, Room, RoomSipConfig, User, utils::is_call_in_allowed,
};
use opentalk_mail_worker_protocol::{MailTask, v1};
use opentalk_types_common::{
    shared_folders::SharedFolder,
    streaming::RoomStreamingTarget,
    tariffs::TariffResource,
    users::{Language, UserId, UserTitle},
};
use snafu::ResultExt;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{Result, metrics::EndpointMetrics};

/// A registered mail recipient
#[derive(Debug)]
pub struct RegisteredMailRecipient {
    /// The user id
    pub id: UserId,
    /// The email address
    pub email: String,
    /// The title
    pub title: UserTitle,
    /// The first name
    pub first_name: String,
    /// The last name
    pub last_name: String,
    /// The language
    pub language: Language,
}

impl RegisteredMailRecipient {
    pub(crate) fn from_inventory_user(user: User, default_user_language: Language) -> Self {
        Self {
            id: user.id,
            email: user.email,
            title: user.title,
            first_name: user.firstname,
            last_name: user.lastname,
            language: user.language.unwrap_or(default_user_language),
        }
    }
}

impl From<RegisteredMailRecipient> for v1::RegisteredUser {
    fn from(
        RegisteredMailRecipient {
            id: _,
            email,
            title,
            first_name,
            last_name,
            language,
        }: RegisteredMailRecipient,
    ) -> Self {
        Self {
            email: email.into(),
            title,
            first_name,
            last_name,
            language,
        }
    }
}

/// An unregistered mail recipient
#[derive(Debug)]
pub struct UnregisteredMailRecipient {
    /// The email address
    pub email: String,
    /// The first name
    pub first_name: String,
    /// The last name
    pub last_name: String,
}

/// An external mail recipient
#[derive(Debug)]
pub struct ExternalMailRecipient {
    /// The email address
    pub email: String,
}

/// A mail recipient
#[derive(Debug)]
pub enum MailRecipient {
    /// A registered mail recipient
    Registered(RegisteredMailRecipient),
    /// An unregistered mail recipient
    Unregistered(UnregisteredMailRecipient),
    /// An external mail recipient
    External(ExternalMailRecipient),
}

fn to_event(
    settings: &Settings,
    event: Event,
    room: Room,
    room_tariff: &TariffResource,
    sip_config: Option<RoomSipConfig>,
    shared_folder: Option<SharedFolder>,
    streaming_targets: Vec<RoomStreamingTarget>,
) -> v1::Event {
    const ONE_DAY_IN_SECONDS: u64 = 86400;

    let created_at = v1::Time {
        time: event.created_at.into(),
        timezone: event.created_at.timezone().to_string(),
    };

    let start_time: Option<v1::Time> = event
        .starts_at()
        .map(DateTime::from)
        .zip(event.starts_at_tz())
        .map(Into::into);

    let end_time: Option<v1::Time> = event
        .ends_at_of_first_occurrence()
        .map(|(timestamp, timezone)| v1::Time::from((DateTime::from(timestamp), timezone)));

    let mut call_in = None;

    if is_call_in_allowed(&room, room_tariff)
        && let (Some(call_in_settings), Some(sip_config)) = (&settings.call_in, sip_config)
    {
        call_in = Some(v1::CallIn {
            sip_tel: call_in_settings.tel.clone(),
            sip_id: sip_config.sip_id.to_string(),
            sip_password: sip_config.password.to_string(),
        });
    }

    let adhoc_retention_seconds = if event.is_adhoc {
        Some(ONE_DAY_IN_SECONDS)
    } else {
        None
    };

    v1::Event {
        id: Uuid::from(event.id),
        rrule: event.recurrence_pattern().map(ToString::to_string),
        created_at,
        start_time,
        end_time,
        room: v1::Room {
            id: Uuid::from(room.id),
            alias: room.alias,
            password: room.password,
        },
        call_in,
        revision: event.revision,
        shared_folder,
        adhoc_retention_seconds,
        streaming_targets,
        // Note: Title and description do not impl Copy, so they partially move
        // event.
        name: event.title,
        description: event.description,
    }
}

fn to_event_exception(exception: EventException) -> v1::EventException {
    let exception_date = v1::Time {
        time: exception.exception_date.into(),
        timezone: exception.exception_date_tz.to_string(),
    };

    let kind = match exception.kind {
        EventExceptionKind::Modified => v1::EventExceptionKind::Modified,
        EventExceptionKind::Cancelled => v1::EventExceptionKind::Canceled,
    };

    let starts_at: Option<v1::Time> = exception
        .starts_at
        .map(DateTime::from)
        .zip(exception.starts_at_tz)
        .map(Into::into);

    let ends_at: Option<v1::Time> = exception
        .ends_at
        .map(DateTime::from)
        .zip(exception.ends_at_tz)
        .map(Into::into);

    v1::EventException {
        exception_date,
        kind,
        title: exception.title,
        description: exception.description,
        is_all_day: exception.is_all_day,
        starts_at,
        ends_at,
    }
}

/// A service for sending emails
#[derive(Clone)]
pub struct MailService {
    metrics: Arc<EndpointMetrics>,
    rabbitmq_pool: Arc<RabbitMqPool>,
    rabbitmq_channel: Arc<Mutex<RabbitMqChannel>>,
}

impl std::fmt::Debug for MailService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MailService")
    }
}

impl MailService {
    const PERSISTENT: u8 = 2;

    /// Creates a new email service
    pub fn new(
        metrics: Arc<EndpointMetrics>,
        rabbitmq_pool: Arc<RabbitMqPool>,
        rabbitmq_channel: RabbitMqChannel,
    ) -> Self {
        Self {
            metrics,
            rabbitmq_pool,
            rabbitmq_channel: Arc::new(Mutex::new(rabbitmq_channel)),
        }
    }

    async fn send_to_rabbitmq(&self, settings: &Settings, mail_task: MailTask) -> Result<()> {
        let Some(rabbitmq_config) = settings.rabbit_mq.as_ref() else {
            return Ok(());
        };
        let Some(queue_name) = rabbitmq_config.mail_task_queue.as_ref() else {
            return Ok(());
        };

        let channel = {
            let mut channel = self.rabbitmq_channel.lock().await;

            if !channel.status().connected() {
                // Check if channel is healthy - try to reconnect if it isn't
                *channel = self
                    .rabbitmq_pool
                    .create_channel()
                    .await
                    .whatever_context("Failed to get a rabbitmq_channel replacement")?;
            }

            channel.clone()
        };

        let mut properties = BasicProperties::default().with_delivery_mode(Self::PERSISTENT);

        if let Some(ttl_milliseconds) = rabbitmq_config.message_ttl_milliseconds() {
            properties = properties.with_expiration(ttl_milliseconds.to_string().into());
        };

        _ = channel
            .basic_publish(
                "",
                queue_name.as_str(),
                Default::default(),
                &serde_json::to_vec(&mail_task)
                    .whatever_context("Failed to serialize mail_task")?,
                properties,
            )
            .await
            .whatever_context("Failed to publish to channel")?;

        self.metrics.increment_issued_email_tasks_count(&mail_task);

        Ok(())
    }

    /// Sends a Registered Invite mail task to the rabbit mq queue, if configured.
    #[expect(clippy::too_many_arguments)]
    pub async fn send_registered_invite(
        &self,
        settings: &Settings,
        inviter: User,
        event: Event,
        room: Room,
        room_tariff: &TariffResource,
        sip_config: Option<RoomSipConfig>,
        invitee: User,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let default_user_language = Language(settings.defaults.user_language.clone());

        let shared_folder = shared_folder.map(|sf| {
            if inviter.id == invitee.id {
                sf
            } else {
                sf.without_write_access()
            }
        });

        // Create MailTask
        let mail_task = MailTask::registered_event_invite(
            RegisteredMailRecipient::from_inventory_user(inviter, default_user_language.clone()),
            to_event(
                settings,
                event,
                room,
                room_tariff,
                sip_config,
                shared_folder,
                streaming_targets,
            ),
            RegisteredMailRecipient::from_inventory_user(invitee, default_user_language),
        );

        self.send_to_rabbitmq(settings, mail_task).await?;
        Ok(())
    }

    /// Sends a Unregistered Invite mail task to the rabbit mq queue, if configured.
    #[expect(clippy::too_many_arguments)]
    pub async fn send_unregistered_invite(
        &self,
        settings: &Settings,
        inviter: User,
        event: Event,
        room: Room,
        room_tariff: &TariffResource,
        sip_config: Option<RoomSipConfig>,
        invitee: opentalk_keycloak_admin::users::User,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let default_user_language = Language(settings.defaults.user_language.clone());

        let invitee = v1::UnregisteredUser {
            email: invitee.email.into(),
            first_name: invitee.first_name.unwrap_or_default(),
            last_name: invitee.last_name.unwrap_or_default(),
        };

        // Create MailTask
        let mail_task = MailTask::unregistered_event_invite(
            RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
            to_event(
                settings,
                event,
                room,
                room_tariff,
                sip_config,
                shared_folder.map(SharedFolder::without_write_access),
                streaming_targets,
            ),
            invitee,
        );

        self.send_to_rabbitmq(settings, mail_task).await?;
        Ok(())
    }

    /// Sends a external Invite mail task to the rabbit mq queue, if configured.
    #[expect(clippy::too_many_arguments)]
    pub async fn send_external_invite(
        &self,
        settings: &Settings,
        inviter: User,
        event: Event,
        room: Room,
        room_tariff: &TariffResource,
        sip_config: Option<RoomSipConfig>,
        invitee: &str,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let default_user_language = Language(settings.defaults.user_language.clone());

        // Create MailTask
        let mail_task = MailTask::external_event_invite(
            RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
            to_event(
                settings,
                event,
                room,
                room_tariff,
                sip_config,
                shared_folder.map(SharedFolder::without_write_access),
                streaming_targets,
            ),
            invitee.to_string(),
        );

        self.send_to_rabbitmq(settings, mail_task).await?;
        Ok(())
    }

    /// Sends an Event Update mail task to the rabbit mq queue, if configured.
    #[expect(clippy::too_many_arguments)]
    pub async fn send_event_update(
        &self,
        settings: &Settings,
        inviter: User,
        event: Event,
        event_exception: Option<EventException>,
        room: Room,
        room_tariff: &TariffResource,
        sip_config: Option<RoomSipConfig>,
        invitee: MailRecipient,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let default_user_language = Language(settings.defaults.user_language.clone());

        let mail_task = match invitee {
            MailRecipient::Registered(invitee) => {
                let shared_folder = shared_folder.map(|sf| {
                    if invitee.id == inviter.id {
                        sf
                    } else {
                        sf.without_write_access()
                    }
                });
                MailTask::registered_event_update(
                    RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                    to_event(
                        settings,
                        event,
                        room,
                        room_tariff,
                        sip_config,
                        shared_folder,
                        streaming_targets,
                    ),
                    event_exception.map(to_event_exception),
                    v1::RegisteredUser {
                        email: v1::Email::new(invitee.email),
                        title: invitee.title,
                        first_name: invitee.first_name,
                        last_name: invitee.last_name,
                        language: invitee.language,
                    },
                )
            }
            MailRecipient::Unregistered(invitee) => MailTask::unregistered_event_update(
                RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                to_event(
                    settings,
                    event,
                    room,
                    room_tariff,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                event_exception.map(to_event_exception),
                v1::UnregisteredUser {
                    email: v1::Email::new(invitee.email),
                    first_name: invitee.first_name,
                    last_name: invitee.last_name,
                },
            ),
            MailRecipient::External(invitee) => MailTask::external_event_update(
                RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                to_event(
                    settings,
                    event,
                    room,
                    room_tariff,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                event_exception.map(to_event_exception),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
            ),
        };

        self.send_to_rabbitmq(settings, mail_task).await?;

        Ok(())
    }

    /// Sends an Event Cancellation mail task to the rabbit mq queue, if configured.
    #[expect(clippy::too_many_arguments)]
    pub async fn send_event_cancellation(
        &self,
        settings: &Settings,
        inviter: User,
        mut event: Event,
        room: Room,
        room_tariff: &TariffResource,
        sip_config: Option<RoomSipConfig>,
        invitee: MailRecipient,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let default_user_language = Language(settings.defaults.user_language.clone());

        // increment event sequence to satisfy icalendar spec
        event.revision += 1;

        let mail_task = match invitee {
            MailRecipient::Registered(invitee) => {
                let shared_folder = shared_folder.map(|sf| {
                    if inviter.id == invitee.id {
                        sf
                    } else {
                        sf.without_write_access()
                    }
                });
                MailTask::registered_event_cancellation(
                    RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                    to_event(
                        settings,
                        event,
                        room,
                        room_tariff,
                        sip_config,
                        shared_folder,
                        streaming_targets,
                    ),
                    v1::RegisteredUser {
                        email: v1::Email::new(invitee.email),
                        title: invitee.title,
                        first_name: invitee.first_name,
                        last_name: invitee.last_name,
                        language: invitee.language,
                    },
                )
            }
            MailRecipient::Unregistered(invitee) => MailTask::unregistered_event_cancellation(
                RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                to_event(
                    settings,
                    event,
                    room,
                    room_tariff,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                v1::UnregisteredUser {
                    email: v1::Email::new(invitee.email),
                    first_name: invitee.first_name,
                    last_name: invitee.last_name,
                },
            ),
            MailRecipient::External(invitee) => MailTask::external_event_cancellation(
                RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                to_event(
                    settings,
                    event,
                    room,
                    room_tariff,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
            ),
        };

        self.send_to_rabbitmq(settings, mail_task).await?;

        Ok(())
    }

    /// Sends an Event Uninvite mail task to the rabbit mq queue, if configured.
    #[expect(clippy::too_many_arguments)]
    pub async fn send_event_uninvite(
        &self,
        settings: &Settings,
        inviter: User,
        mut event: Event,
        room: Room,
        room_tariff: &TariffResource,
        sip_config: Option<RoomSipConfig>,
        invitee: MailRecipient,
        shared_folder: Option<SharedFolder>,
        streaming_targets: Vec<RoomStreamingTarget>,
    ) -> Result<()> {
        let default_user_language = Language(settings.defaults.user_language.clone());

        // increment event sequence to satisfy icalendar spec
        event.revision += 1;

        let mail_task = match invitee {
            MailRecipient::Registered(invitee) => {
                let shared_folder = shared_folder.map(|sf| {
                    if inviter.id == invitee.id {
                        sf
                    } else {
                        sf.without_write_access()
                    }
                });
                MailTask::registered_event_uninvite(
                    RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                    to_event(
                        settings,
                        event,
                        room,
                        room_tariff,
                        sip_config,
                        shared_folder,
                        streaming_targets,
                    ),
                    v1::RegisteredUser {
                        email: v1::Email::new(invitee.email),
                        title: invitee.title,
                        first_name: invitee.first_name,
                        last_name: invitee.last_name,
                        language: invitee.language,
                    },
                )
            }
            MailRecipient::Unregistered(invitee) => MailTask::unregistered_event_uninvite(
                RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                to_event(
                    settings,
                    event,
                    room,
                    room_tariff,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                v1::UnregisteredUser {
                    email: v1::Email::new(invitee.email),
                    first_name: invitee.first_name,
                    last_name: invitee.last_name,
                },
            ),
            MailRecipient::External(invitee) => MailTask::external_event_uninvite(
                RegisteredMailRecipient::from_inventory_user(inviter, default_user_language),
                to_event(
                    settings,
                    event,
                    room,
                    room_tariff,
                    sip_config,
                    shared_folder.map(SharedFolder::without_write_access),
                    streaming_targets,
                ),
                v1::ExternalUser {
                    email: v1::Email::new(invitee.email),
                },
            ),
        };

        self.send_to_rabbitmq(settings, mail_task).await?;

        Ok(())
    }
}
