// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use chrono::Local;
use chrono_tz::Tz;
use icu_locid::{LanguageIdentifier, langid};
use opentalk_report_generation::{GenerateOptions, ToReportDateTime};
use opentalk_roomserver_room::storage::assets::{AssetUploaded, ModuleAssetStorage};
use opentalk_roomserver_signaling::{
    localization,
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, NoOp, NoSettings, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
    storage::assets::AssetMetaData,
};
use opentalk_roomserver_types::{
    connection_id::ConnectionId, room_parameters::EventContext,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_meeting_report::{
    MEETING_REPORT_MODULE_ID,
    command::MeetingReportCommand,
    event::{MeetingReportError, MeetingReportEvent},
};
use opentalk_types_common::{
    assets::{AssetFileKind, FileExtension, asset_file_kind},
    events::{EventDescription, EventTitle},
    modules::ModuleId,
    time::{TimeZone, Timestamp},
};
use opentalk_types_signaling::ParticipantId;

use crate::template::{ReportParticipant, ReportTemplateParameter};

mod template;

pub struct MeetingReportModule {
    typst_packages_path: PathBuf,
}

const TEMPLATE: &str = include_str!("../templates/attendance_report.typ");
const FTL_EN: &str = include_str!("../templates/l10n/en.ftl");
const FTL_DE: &str = include_str!("../templates/l10n/de.ftl");
const AVAILABLE_LANGUAGES: &[LanguageIdentifier] = &[langid!("en"), langid!("de")];

impl SignalingModuleDescription for MeetingReportModule {
    const MODULE_ID: ModuleId = MEETING_REPORT_MODULE_ID;
    const DESCRIPTION: &'static str =
        "Handles generation of meeting reports, e.g. participant list export";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for MeetingReportModule {
    const NAMESPACE: ModuleId = MEETING_REPORT_MODULE_ID;

    type Incoming = MeetingReportCommand;

    type Outgoing = MeetingReportEvent;

    type Internal = NoOp;

    type Loopback = Result<AssetUploaded, SignalingModuleError<MeetingReportError>>;

    type JoinInfo = ();

    type PeerJoinInfo = ();

    type Error = MeetingReportError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let typst_packages_path = init_data.settings.reports.typst.packages_path.clone();
        Some(Self {
            typst_packages_path,
        })
    }

    #[allow(unused_variables)]
    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        Ok(ModuleJoinData::default())
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            MeetingReportCommand::GenerateAttendanceReport {
                include_email_addresses,
            } => self.generate_attendance_report(ctx, sender, include_email_addresses)?,
        }
        Ok(())
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let uploaded = event?;
        tracing::debug!(
            "Generated meeting attendance report: {}({})",
            uploaded.filename,
            uploaded.id
        );
        ctx.send_ws_message(
            ctx.participants.connected().ids(),
            MeetingReportEvent::PdfAsset {
                filename: uploaded.filename,
                asset_id: uploaded.id,
            },
        )?;
        Ok(())
    }
}

impl MeetingReportModule {
    fn generate_attendance_report(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        include_email_addresses: bool,
    ) -> Result<(), SignalingModuleError<MeetingReportError>> {
        if !ctx.is_moderator(sender) {
            return Err(MeetingReportError::InsufficientPermissions.into());
        }

        let storage = ctx.assets();
        let report_timezone = ctx.room_task_info.room.created_by.timezone;
        let language = localization::negotiate_languages(ctx, AVAILABLE_LANGUAGES)
            .ok_or(MeetingReportError::Generate)?;
        let tz = Tz::from(report_timezone);

        let participants = ctx
            .participants
            .visible()
            .iter()
            .map(|(id, state)| ReportParticipant {
                id: *id,
                name: state.display_name.to_string(),
                role: state.role,
                email: include_email_addresses
                    .then(|| state.kind.email())
                    .flatten()
                    .map(String::from)
                    .unwrap_or_default(),
                joined_at: state.joined_at.to_report_date_time(&tz),
                left_at: state.left_at.to_report_date_time(&tz),
            })
            .collect();

        let event = ctx.room_task_info.room.event.clone();
        let typst_package_path = self.typst_packages_path.clone();

        ctx.spawn(async move {
            Self::generate_report(
                storage,
                report_timezone,
                language,
                tz,
                participants,
                event,
                typst_package_path,
            )
            .await
        });

        Ok(())
    }

    fn generate_pdf_report_from_template(
        template: String,
        parameter: &ReportTemplateParameter,
        typst_package_path: &Path,
    ) -> Result<Vec<u8>, SignalingModuleError<MeetingReportError>> {
        let mut generate_options = GenerateOptions::default();
        generate_options.packages_path = Some(typst_package_path);

        let serialized = serde_json::to_string(parameter)
            .inspect_err(|e| tracing::error!("Failed to serialize parameters: {e:?}"))
            .map_err(|_| MeetingReportError::Generate)?
            .into_bytes()
            .into();
        let files = [
            ("data.json", (None, serialized)),
            ("l10n/de.ftl", (None, FTL_DE.as_bytes().into())),
            ("l10n/en.ftl", (None, FTL_EN.as_bytes().into())),
        ];

        let report = opentalk_report_generation::generate_pdf_report(
            template,
            BTreeMap::from_iter(files),
            &generate_options,
        )
        .map_err(|err| {
            tracing::error!("Failed to generate report: {err}");
            MeetingReportError::Generate
        })?;

        Ok(report)
    }

    async fn generate_report(
        storage: ModuleAssetStorage,
        report_timezone: TimeZone,
        report_language: LanguageIdentifier,
        tz: Tz,
        participants: Vec<ReportParticipant>,
        event: Option<EventContext>,
        typst_package_path: PathBuf,
    ) -> Result<AssetUploaded, SignalingModuleError<MeetingReportError>> {
        const ASSET_FILE_KIND: AssetFileKind = asset_file_kind!("meeting_report");

        if !storage.can_upload().await {
            return Err(MeetingReportError::StorageExceeded.into());
        }

        let event = event.as_ref();
        let starts_at = event
            .and_then(|event| event.starts_at)
            .to_report_date_time(&tz);
        let ends_at = event
            .and_then(|event| event.ends_at)
            .to_report_date_time(&tz);
        let available_languages = AVAILABLE_LANGUAGES.to_vec();
        let title = event.map_or_else(
            || EventTitle::from_str_lossy(""),
            |event| event.title.clone(),
        );
        let description = event.map_or_else(
            || EventDescription::from_str_lossy(""),
            |event| event.description.clone(),
        );
        let current_time = Local::now();
        let report_created_at = current_time.to_report_date_time(&tz);
        let report = Self::generate_pdf_report_from_template(
            TEMPLATE.to_string(),
            &ReportTemplateParameter {
                available_languages,
                title,
                description,
                starts_at,
                ends_at,
                report_created_at,
                report_timezone,
                report_language,
                participants,
            },
            &typst_package_path,
        )?;

        let upload = storage
            .upload_asset_vec(
                report,
                AssetMetaData {
                    kind: ASSET_FILE_KIND,
                    timestamp: Timestamp::now(),
                    extension: FileExtension::pdf(),
                },
            )
            .await
            .map_err(|e| {
                tracing::error!("Failed to upload meeting report: {e:?}");
                MeetingReportError::from(e)
            })?;
        Ok(upload)
    }
}

#[cfg(test)]
mod tests {
    use icu_locid::langid;
    use insta::assert_snapshot;
    use opentalk_roomserver_common::settings::runtime_settings::reports_typst::reports_typst_packages_test_path;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::{MeetingReportModule, TEMPLATE, template::ReportTemplateParameter};

    fn generate(parameter: &ReportTemplateParameter) -> String {
        let typst_packages_path = reports_typst_packages_test_path();
        assert!(
            typst_packages_path.exists(),
            "Please make sure that the typst packages path {typst_packages_path:?} exists and contains the required typst packages, or point the TYPST_PACKAGE_CACHE_PATH environment variable to the path with the typst packages"
        );

        let pdf = MeetingReportModule::generate_pdf_report_from_template(
            TEMPLATE.to_string(),
            parameter,
            &typst_packages_path,
        )
        .expect("generation should work");
        pdf_extract::extract_text_from_mem(&pdf)
            .expect("text should be extractable from generated pdf")
    }

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&MeetingReportModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[meeting_report]");
    }

    #[test_log::test]
    fn generate_report_small() {
        insta::with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}", "[timestamp]")
        ]}, {
            assert_snapshot!(generate(&crate::template::tests::example_small()), @r"
            Attendance Report
             Meeting : Testmeeting

            Report created at : [timestamp]

            Report timezone : Europe/Berlin

            Participants
             Nr Name Role

            1 Alice Adams Moderator
            ")
        });
    }

    #[test_log::test]
    fn generate_report_small_de() {
        let mut report_template_data = crate::template::tests::example_small();
        report_template_data.report_language = langid!("de");

        insta::with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}", "[timestamp]")
        ]}, {
            assert_snapshot!(generate(&report_template_data), @r"
            Anwesenheitsbericht
             Meeting : Testmeeting

            Bericht erstellt um : [timestamp]

            Zeitzone des Berichts : Europe/Berlin

            Teilnehmende
             Nr Name Rolle

            1 Alice Adams Moderator
            ")
        });
    }

    #[test_log::test]
    fn generate_report_medium() {
        insta::with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}", "[timestamp]")
        ]}, {
            assert_snapshot!(generate(&crate::template::tests::example_medium()), @r"
            Attendance Report
             Meeting : Testmeeting

            Details : A medium sized test meeting

            Planned start : [timestamp]

            Planned end : [timestamp]

            Report created at : [timestamp]

            Report timezone : Europe/Berlin

            Participants
             Nr Name Role

            1 Alice Adams Moderator

            2 Charlie Cooper User

            3 Bob Burton User
            ")
        });
    }

    #[test_log::test]
    fn generate_report_medium_de() {
        let mut report_template_data = crate::template::tests::example_medium();
        report_template_data.report_language = langid!("de");

        insta::with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}", "[timestamp]")
        ]}, {
            assert_snapshot!(generate(&report_template_data), @r"
            Anwesenheitsbericht
             Meeting : Testmeeting

            Details : A medium sized test meeting

            Geplanter Beginn : [timestamp]

            Geplantes Ende : [timestamp]

            Bericht erstellt um : [timestamp]

            Zeitzone des Berichts : Europe/Berlin

            Teilnehmende
             Nr Name Rolle

            1 Alice Adams Moderator

            2 Charlie Cooper Nutzer

            3 Bob Burton Nutzer
            ")
        });
    }

    #[test_log::test]
    fn generate_report_large() {
        insta::with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}", "[timestamp]")
        ]}, {
            assert_snapshot!(generate(&crate::template::tests::example_large()), @r"
            Attendance Report
             Meeting : Large Testmeeting

            Details : The large test meeting

            Planned start : [timestamp]

            Planned end : [timestamp]

            Report created at : [timestamp]

            Report timezone : Europe/Berlin

            Participants
             Nr Name Role

            1 Alice Adams Moderator

            2 Franz Fischer User

            3 Recorder User

            4 Charlie Cooper User

            5 Bob Burton User

            6 Erin Guest

            7 Dave Dunn Guest
            ")
        });
    }

    #[test_log::test]
    fn generate_report_large_de() {
        let mut report_template_data = crate::template::tests::example_large();
        report_template_data.report_language = langid!("de");

        insta::with_settings!({filters => vec![
            (r"[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}", "[timestamp]")
        ]}, {
            assert_snapshot!(generate(&report_template_data), @r"
            Anwesenheitsbericht
             Meeting : Large Testmeeting

            Details : The large test meeting

            Geplanter Beginn : [timestamp]

            Geplantes Ende : [timestamp]

            Bericht erstellt um : [timestamp]

            Zeitzone des Berichts : Europe/Berlin

            Teilnehmende
             Nr Name Rolle

            1 Alice Adams Moderator

            2 Franz Fischer Nutzer

            3 Recorder Nutzer

            4 Charlie Cooper Nutzer

            5 Bob Burton Nutzer

            6 Erin Gast

            7 Dave Dunn Gast
            ")
        });
    }
}
