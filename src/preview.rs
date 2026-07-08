// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{str::FromStr as _, time::Duration};

use anyhow::Context;
use clap::ValueEnum;
use lettre::{AsyncSmtpTransport, Tokio1Executor};
use opentalk_mail_worker_protocol::{
    self as protocol,
    v1::{
        ExternalEventCancellation, ExternalEventInvite, RegisteredEventCancellation,
        RegisteredEventInvite, RegisteredEventUninvite, RegisteredEventUpdate,
        UnregisteredEventCancellation, UnregisteredEventInvite,
    },
};
use opentalk_types_common::{
    rooms::RoomPassword,
    shared_folders::{SharedFolder, SharedFolderAccess},
    streaming::{RoomStreamingTarget, StreamingKey, StreamingTarget, StreamingTargetKind},
    users::{Language, UserTitle},
    utils::ExampleData as _,
};
use protocol::v1::{CallIn, Event, Room, Time};
use uuid::Uuid;

use crate::{MailBuilder, MailTemplate, send_mail_v1, settings::Settings};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputVariant {
    Html,
    Plain,
}

impl OutputVariant {
    /// Returns `true` if the output variant is [`Html`].
    ///
    /// [`Html`]: OutputVariant::Html
    #[must_use]
    fn is_html(&self) -> bool {
        matches!(self, Self::Html)
    }
}

impl From<&OutputVariant> for bool {
    fn from(val: &OutputVariant) -> Self {
        match val {
            OutputVariant::Html => true,
            OutputVariant::Plain => false,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[allow(clippy::enum_variant_names)]
pub enum TemplateVariant {
    RegisteredInvite,
    RegisteredEventUpdate,
    RegisteredCancellation,
    RegisteredUninvite,
    UnregisteredInvite,
    UnregisteredCancellation,
    ExternalInvite,
    ExternalCancellation,
}

impl TemplateVariant {
    pub fn render_template(
        &self,
        settings: &Settings,
        output: OutputVariant,
        language: Language,
    ) -> anyhow::Result<String> {
        let example_mail = match self {
            TemplateVariant::RegisteredInvite => {
                RegisteredEventInvite::preview(settings, output.is_html(), language)
            }
            TemplateVariant::RegisteredEventUpdate => {
                RegisteredEventUpdate::preview(settings, output.is_html(), language)
            }
            TemplateVariant::RegisteredCancellation => {
                RegisteredEventCancellation::preview(settings, output.is_html(), language)
            }
            TemplateVariant::RegisteredUninvite => {
                RegisteredEventUninvite::preview(settings, output.is_html(), language)
            }
            TemplateVariant::UnregisteredInvite => {
                UnregisteredEventInvite::preview(settings, output.is_html(), language)
            }
            TemplateVariant::UnregisteredCancellation => {
                UnregisteredEventCancellation::preview(settings, output.is_html(), language)
            }
            TemplateVariant::ExternalInvite => {
                ExternalEventInvite::preview(settings, output.is_html(), language)
            }
            TemplateVariant::ExternalCancellation => {
                ExternalEventCancellation::preview(settings, output.is_html(), language)
            }
        }
        .context("Failed to create preview")?;
        Ok(example_mail)
    }
}

const EVENT_DESCRIPTION_REGISTERED: &str = "This event is a dummy event, and you should have got this invite as a registered user. You can safely ignore this description";
const EVENT_DESCRIPTION_EXTERNAL: &str = "This event is a dummy event, and you should have got this invite as a external user. You can safely ignore this description";
const EVENT_DESCRIPTION_UNREGISTERED: &str = "This event is a dummy event, and you should have got this invite as a keycloak user. You can safely ignore this description";

fn build_shared_folder() -> Option<SharedFolder> {
    Some(SharedFolder {
        read: SharedFolderAccess {
            url: "https://www.example.org/read".to_owned(),
            password: "ReadSecret".to_owned(),
        },
        read_write: Some(SharedFolderAccess {
            url: "https://www.example.org/write".to_owned(),
            password: "WriteSecret".to_owned(),
        }),
    })
}

fn unregistered_invitee(email_address: &Option<String>) -> protocol::v1::UnregisteredUser {
    protocol::v1::UnregisteredUser {
        email: email_address
            .as_deref()
            .unwrap_or("invitee@example.org")
            .into(),
        first_name: "Ingrid".to_string(),
        last_name: "Invitee".to_string(),
    }
}

fn external_invitee(email_address: &Option<String>) -> protocol::v1::ExternalUser {
    protocol::v1::ExternalUser {
        email: email_address
            .as_deref()
            .unwrap_or("external.invitee@example.org")
            .into(),
    }
}

fn registered_invitee(
    language: Language,
    email_address: &Option<String>,
) -> protocol::v1::RegisteredUser {
    protocol::v1::RegisteredUser {
        email: email_address
            .as_deref()
            .unwrap_or("invitee@example.org")
            .into(),
        first_name: "Ingrid".to_string(),
        last_name: "Invitee".to_string(),
        title: UserTitle::from_str("Prof.").expect("Prof. is a valid user title"),
        language,
    }
}

fn registered_inviter(
    language: Language,
    email_address: &Option<String>,
) -> protocol::v1::RegisteredUser {
    protocol::v1::RegisteredUser {
        email: email_address
            .as_deref()
            .unwrap_or("ernest.inviter@example.org")
            .into(),
        first_name: "Ernest".to_string(),
        last_name: "Inviter".to_string(),
        title: UserTitle::example_data(),
        language,
    }
}

pub trait ExampleData: Sized + MailTemplate {
    fn generate_example(language: Language, email_address: &Option<String>)
    -> anyhow::Result<Self>;

    fn preview(settings: &Settings, html: bool, lang: Language) -> anyhow::Result<String> {
        let mail_builder =
            MailBuilder::new(settings).context("Failed to initialize MailBuilder")?;

        let demo_message = Self::generate_example(lang, &None)?;

        if html {
            return demo_message
                .generate_email_html(&mail_builder)
                .context("Failed to generate email html");
        }

        demo_message
            .generate_email_plain(&mail_builder)
            .context("Failed to generate email plain")
    }
}

fn generate_example_event(description: String) -> anyhow::Result<Event> {
    let start_date = chrono::DateTime::parse_from_rfc3339("2024-10-10 15:30:00+02:00")
        .expect("Must be valid date")
        .to_utc();

    let created_at = chrono::DateTime::parse_from_rfc3339("2024-10-04 13:30:25+02:00")
        .expect("Must be valid date")
        .to_utc();

    Ok(Event {
        id: Uuid::from_u128(0x7dfb7d8d_fb57_49ba_aacc_a76b8d390000),
        name: "This is a Preview Event"
            .parse()
            .expect("Example must be valid"),
        description: description.parse()?,
        room: Room {
            id: Uuid::from_u128(0x7dfb7d8d_fb57_49ba_aacc_a76b8d390001),
            alias: None,
            password: Some(
                RoomPassword::from_str("password123")
                    .context("Example room password was invalid")?,
            ),
        },
        created_at: Time {
            time: created_at,
            timezone: "Europe/Berlin".into(),
        },
        start_time: Some(Time {
            time: start_date,
            timezone: "Europe/Berlin".into(),
        }),
        end_time: Some(Time {
            time: start_date + chrono::Duration::minutes(40),
            timezone: "Europe/Berlin".into(),
        }),
        rrule: None,
        call_in: Some(CallIn {
            sip_tel: "+4930405051330".into(),
            sip_id: "1234567890".into(),
            sip_password: "1234567890".into(),
        }),
        revision: 0,
        shared_folder: build_shared_folder(),
        adhoc_retention_seconds: None,
        streaming_targets: vec![
            RoomStreamingTarget {
                id: Uuid::from_u128(0x7dfb7d8d_fb57_49ba_aacc_a76b8d390002).into(),
                streaming_target: StreamingTarget {
                    name: "streaming service 1".to_string(),
                    kind: StreamingTargetKind::Custom {
                        streaming_endpoint: "https://stream-a.example.com"
                            .parse()
                            .expect("This is a valid URL!"),
                        streaming_key: StreamingKey::from("value".to_string()),
                        public_url: "https://example.com/4385f873-6077-4d2c-8b46-dbec62029ac6"
                            .parse()
                            .expect("This is a valid URL!"),
                    },
                },
            },
            RoomStreamingTarget {
                id: Uuid::from_u128(0x7dfb7d8d_fb57_49ba_aacc_a76b8d390003).into(),
                streaming_target: StreamingTarget {
                    name: "streaming service 2".to_string(),
                    kind: StreamingTargetKind::Custom {
                        streaming_endpoint: "https://stream-b.example.com"
                            .parse()
                            .expect("This is a valid URL!"),
                        streaming_key: StreamingKey::from("value".to_string()),
                        public_url: "https://example.com/f544bef8-aa33-475b-8664-fa17c5ca7f83"
                            .parse()
                            .expect("This is a valid URL!"),
                    },
                },
            },
        ],
    })
}

impl ExampleData for protocol::v1::UnregisteredEventInvite {
    fn generate_example(
        language: Language,
        email_address: &Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            invitee: unregistered_invitee(email_address),
            event: generate_example_event(EVENT_DESCRIPTION_UNREGISTERED.to_string())?,
            inviter: registered_inviter(language, &None),
        })
    }
}

impl ExampleData for protocol::v1::UnregisteredEventCancellation {
    fn generate_example(
        language: Language,
        email_address: &Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            invitee: unregistered_invitee(email_address),
            event: generate_example_event(EVENT_DESCRIPTION_UNREGISTERED.to_string())?,
            inviter: registered_inviter(language, &None),
        })
    }
}

impl ExampleData for protocol::v1::RegisteredEventInvite {
    fn generate_example(
        language: Language,
        email_address: &Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            invitee: registered_invitee(language.clone(), email_address),
            event: generate_example_event(EVENT_DESCRIPTION_REGISTERED.to_string())?,
            inviter: registered_inviter(language, email_address),
        })
    }
}

impl ExampleData for protocol::v1::RegisteredEventUninvite {
    fn generate_example(
        language: Language,
        email_address: &Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            invitee: registered_invitee(language.clone(), email_address),
            event: generate_example_event(EVENT_DESCRIPTION_REGISTERED.to_string())?,
            inviter: registered_inviter(language, email_address),
        })
    }
}

impl ExampleData for protocol::v1::RegisteredEventUpdate {
    fn generate_example(
        language: Language,
        email_address: &Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            invitee: registered_invitee(language.clone(), email_address),
            event: generate_example_event(EVENT_DESCRIPTION_REGISTERED.to_string())?,
            inviter: registered_inviter(language, email_address),
            event_exception: None,
        })
    }
}

impl ExampleData for protocol::v1::RegisteredEventCancellation {
    fn generate_example(
        language: Language,
        email_address: &Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            invitee: registered_invitee(language.clone(), email_address),
            event: generate_example_event(EVENT_DESCRIPTION_REGISTERED.to_string())?,
            inviter: registered_inviter(language, &None),
        })
    }
}

impl ExampleData for protocol::v1::ExternalEventInvite {
    fn generate_example(
        language: Language,
        email_address: &Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            invitee: external_invitee(email_address),
            event: generate_example_event(EVENT_DESCRIPTION_EXTERNAL.to_string())?,
            inviter: registered_inviter(language, email_address),
        })
    }
}

impl ExampleData for protocol::v1::ExternalEventCancellation {
    fn generate_example(
        language: Language,
        email_address: &Option<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            invitee: external_invitee(email_address),
            event: generate_example_event(EVENT_DESCRIPTION_EXTERNAL.to_string())?,
            inviter: registered_inviter(language, &None),
        })
    }
}

pub async fn preview_send_mail(
    settings: &Settings,
    template: TemplateVariant,
    to: String,
    cancellation_delay: u64,
) -> anyhow::Result<()> {
    let smtp_client: AsyncSmtpTransport<Tokio1Executor> =
        settings.smtp.smtp_server.clone().try_into()?;
    let mail_builder = MailBuilder::new(settings)?;

    let duration_between_invite_and_cancellation = Duration::from_secs(cancellation_delay);
    let receiver = Some(to);
    let lang: Language =
        Language::from_str("en-US").map_err(|e| anyhow::anyhow!("Invalid language. {}", e))?;

    let message = match template {
        TemplateVariant::RegisteredInvite => protocol::v1::Message::RegisteredEventInvite(
            RegisteredEventInvite::generate_example(lang, &receiver)
                .context("Failed to generate example RegisteredEventInvite")?,
        ),
        TemplateVariant::UnregisteredInvite => protocol::v1::Message::UnregisteredEventInvite(
            UnregisteredEventInvite::generate_example(lang, &receiver)
                .context("Failed to generate example UnregisteredEventInvite")?,
        ),
        TemplateVariant::ExternalInvite => protocol::v1::Message::ExternalEventInvite(
            ExternalEventInvite::generate_example(lang, &receiver)
                .context("Failed to generate example ExternalEventInvite")?,
        ),
        TemplateVariant::RegisteredCancellation => {
            let invite = RegisteredEventInvite::generate_example(lang.clone(), &receiver)
                .context("Failed to generate example RegisteredEventInvite")?;
            let event_id = invite.event.id;

            send_mail_v1(
                &smtp_client,
                &mail_builder,
                &protocol::v1::Message::RegisteredEventInvite(invite),
            )
            .await
            .context("Failed to send mail")?;

            tokio::time::sleep(duration_between_invite_and_cancellation).await;

            let mut cancellation =
                RegisteredEventCancellation::generate_example(lang, &receiver)
                    .context("Failed to generate example RegisteredEventCancellation")?;
            cancellation.event.id = event_id;
            protocol::v1::Message::RegisteredEventCancellation(cancellation)
        }
        TemplateVariant::UnregisteredCancellation => {
            let invite = UnregisteredEventInvite::generate_example(lang.clone(), &receiver)
                .context("Failed to generate example UnregisteredEventInvite")?;
            let event_id = invite.event.id;

            send_mail_v1(
                &smtp_client,
                &mail_builder,
                &protocol::v1::Message::UnregisteredEventInvite(invite),
            )
            .await
            .context("Failed to send mail")?;

            tokio::time::sleep(duration_between_invite_and_cancellation).await;

            let mut cancellation = UnregisteredEventCancellation::generate_example(lang, &receiver)
                .context("Failed to generate example UnregisteredEventCancellation")?;
            cancellation.event.id = event_id;
            protocol::v1::Message::UnregisteredEventCancellation(cancellation)
        }
        TemplateVariant::ExternalCancellation => {
            let invite = ExternalEventInvite::generate_example(lang.clone(), &receiver)
                .context("Failed to generate example ExternalEventInvite")?;
            let event_id = invite.event.id;

            send_mail_v1(
                &smtp_client,
                &mail_builder,
                &protocol::v1::Message::ExternalEventInvite(invite),
            )
            .await
            .context("Failed to send mail")?;

            tokio::time::sleep(duration_between_invite_and_cancellation).await;

            let mut cancellation = ExternalEventCancellation::generate_example(lang, &receiver)
                .context("Failed to generate example ExternalEventCancellation")?;
            cancellation.event.id = event_id;
            protocol::v1::Message::ExternalEventCancellation(cancellation)
        }
        TemplateVariant::RegisteredEventUpdate => protocol::v1::Message::RegisteredEventUpdate(
            RegisteredEventUpdate::generate_example(lang, &receiver)
                .context("Failed to generate example RegisteredEventUpdate")?,
        ),
        TemplateVariant::RegisteredUninvite => protocol::v1::Message::RegisteredEventUninvite(
            RegisteredEventUninvite::generate_example(lang, &receiver)
                .context("Failed to generate example RegisteredEventUninvite")?,
        ),
    };

    send_mail_v1(&smtp_client, &mail_builder, &message)
        .await
        .context("Failed to send mail")?;

    Ok(())
}
