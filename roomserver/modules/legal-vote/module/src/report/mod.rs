// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! The generated PDF reports contain two invisible Unicode characters U+2068 (First Strong Isolate)
//! and U+2069 (Pop Directional Isolate). This is done by fluent to ensure correct rendering of
//! user-provided text in both left-to-right and right-to-left languages. For details, see [the
//! fluent documentation on this matter](https://github.com/projectfluent/fluent.js/wiki/Unicode-Isolation).
//! Because of this, the content of the generated PDFs cannot be inlined in insta snapshots.

pub mod data;

use data::ReportData;
pub use error::Error;
use icu_locid::{LanguageIdentifier, langid};
use opentalk_report_generation::GenerateOptions;
use opentalk_types_common::users::{DisplayName, UserId};

mod error;
mod report_data_builder;

use std::{collections::BTreeMap, path::Path};

use chrono_tz::Tz;
use report_data_builder::Builder;

use crate::protocol::v1::ProtocolEntry;

const TEMPLATE: &str = include_str!("../../templates/legal_vote_report.typ");
const FTL_EN: &str = include_str!("../../templates/l10n/en.ftl");
const FTL_DE: &str = include_str!("../../templates/l10n/de.ftl");
pub(super) const AVAILABLE_LANGUAGES: &[LanguageIdentifier] = &[langid!("en"), langid!("de")];

pub(crate) fn generate(
    user_names: BTreeMap<UserId, DisplayName>,
    protocol: Vec<ProtocolEntry>,
    timezone: Tz,
    report_language: LanguageIdentifier,
    typst_package_path: &Path,
) -> Result<Vec<u8>, Error> {
    let builder = Builder::new(user_names);
    let report_data = builder.build_report_data(protocol, timezone, report_language)?;

    generate_from_template(TEMPLATE.to_string(), &report_data, typst_package_path)
}

fn generate_from_template(
    template: String,
    parameter: &ReportData,
    typst_package_path: &Path,
) -> Result<Vec<u8>, Error> {
    let mut generate_options = GenerateOptions::default();
    generate_options.packages_path = Some(typst_package_path);

    let serialized = serde_json::to_string_pretty(parameter)
        .unwrap()
        .into_bytes()
        .into();
    let files = [
        ("data.json", (None, serialized)),
        ("l10n/en.ftl", (None, FTL_EN.as_bytes().into())),
        ("l10n/de.ftl", (None, FTL_DE.as_bytes().into())),
    ];

    Ok(opentalk_report_generation::generate_pdf_report(
        template,
        BTreeMap::from_iter(files),
        &generate_options,
    )?)
}

#[cfg(test)]
mod tests {
    use icu_locid::langid;
    use insta::assert_snapshot;
    use opentalk_roomserver_common::settings::runtime_settings::reports_typst::reports_typst_packages_test_path;

    use super::{
        TEMPLATE,
        data::{
            ReportData,
            report_data::tests::{example_pseudonymous, example_public},
        },
        generate_from_template,
    };
    use crate::report::data::report_data::tests::{
        canceled_public, initiator_left_public, room_destroyed_public,
    };

    fn generate(parameter: &ReportData) -> String {
        let typst_packages_path = reports_typst_packages_test_path();
        assert!(
            typst_packages_path.exists(),
            "Please make sure that the typst packages path {typst_packages_path:?} exists and contains the required typst packages, or point the TYPST_PACKAGE_CACHE_PATH environment variable to the path with the typst packages"
        );

        let pdf = generate_from_template(TEMPLATE.to_string(), parameter, &typst_packages_path)
            .expect("generation should work");
        pdf_extract::extract_text_from_mem(&pdf)
            .expect("text should be extractable from generated pdf")
    }

    #[test]
    fn generate_report_live_roll_call() {
        assert_snapshot!(
            generate(&example_public()),
            @r"


        OpenTalk Vote Report
         Title : Weather Vote

        Subtitle : Another one of these weather votes

        Topic : Is the weather good today?

        Pseudonymous : No

        Referendum leader : Alice Adams

        Vote id : ee621ab4-72f6-4d39-bbc4-dc1b96a606cf

        Start : 2025-01-02 03:04:05

        End : 2025-01-02 03:09:05

        Report timezone : Europe/Berlin

        Participant count : 8

        Scheduled duration : 300 s

        Abstention : Disallowed

        Automatic close : Enabled

        Vote ended due to : All users voted

        Number of votes : 6

        Results
         Vote Count

        Approval 4

        Disapproval 2

        Abstention 5

        Recorded votes
         Name Token Vote Timestamp

        Alice Adams aaaaaaaa Approval 2025-01-02 03:04:24

        Bob Burton bbbbbbbb Disapproval 2025-01-02 03:04:20

        Charlie Cooper cccccccc Disapproval 2025-01-02 03:04:21

        Dave Dunn dddddddd Approval 2025-01-02 03:04:19

        Erin Eaton eeeeeeee Approval 2025-01-02 03:06:00

        George Grump gggggggg Approval 2025-01-02 03:06:00

        Event log
         Name Timestamp Event

        Charlie Cooper 2025-01-02 03:04:18 Reports a screenshare issue
        "
        );
    }

    #[test]
    fn generate_report_live_roll_call_de() {
        let mut report_data = example_public();
        report_data.report_language = langid!("de");

        assert_snapshot!(
            generate(&report_data),
            @r"


        OpenTalk Abstimmungsbericht
         Titel : Weather Vote

        Untertitel : Another one of these weather votes

        Thema : Is the weather good today?

        Pseudonym : Nein

        Abstimmungsleitung : Alice Adams

        Abstimmungs-ID : ee621ab4-72f6-4d39-bbc4-dc1b96a606cf

        Beginn : 2025-01-02 03:04:05

        Ende : 2025-01-02 03:09:05

        Zeitzone des Berichts : Europe/Berlin

        Anzahl Teilnehmende : 8

        Geplante Dauer : 300 s

        Enthaltung : nicht zulässig

        Automatisches Ende : aktiviert

        Abstimmung endete auf Grund von : Alle Stimmen abgegeben

        Anzahl abgegebener Stimmen : 6

        Ergebnisse
         Stimme Anzahl

        Zustimmung 4

        Ablehnung 2

        Enthaltung 5

        Recorded votes
         Name Token Stimme Zeitpunkt

        Alice Adams aaaaaaaa Zustimmung 2025-01-02 03:04:24

        Bob Burton bbbbbbbb Ablehnung 2025-01-02 03:04:20

        Charlie Cooper cccccccc Ablehnung 2025-01-02 03:04:21

        Dave Dunn dddddddd Zustimmung 2025-01-02 03:04:19

        Erin Eaton eeeeeeee Zustimmung 2025-01-02 03:06:00

        George Grump gggggggg Zustimmung 2025-01-02 03:06:00

        Ereignisprotokoll
         Name Zeitpunkt Ereignis

        Charlie Cooper 2025-01-02 03:04:18 Meldet ein Screenshare-Problem
        "
        );
    }

    #[test]
    fn generate_pseudonymous() {
        assert_snapshot!(generate(&example_pseudonymous()),
        @r"


        OpenTalk Vote Report
         Title : Example Pseudonymous Vote

        Pseudonymous : Yes

        Referendum leader : Alice Adams

        Vote id : 6a3525fc-aeef-4d7e-9d76-e41ab2cbe469

        Start : 2025-02-08 12:32:09

        End : 2025-02-08 12:32:22

        Report timezone : Europe/Vienna

        Participant count : 4

        Scheduled duration : 60 s

        Abstention : Allowed

        Automatic close : Enabled

        Vote ended due to : All users voted

        Number of votes : 4

        Results
         Vote Count

        Approval 1

        Disapproval 2

        Abstention 1

        Recorded votes
         Name Token Vote Timestamp

        Hidden LPwNXJWs7b1 Approval —

        Hidden K5SMSt98f11 Disapproval —

        Hidden B1yWM5eWQQi Abstention —

        Hidden 8PCkuJ9NGoY Disapproval —

        Event log
         Name Timestamp Event
        ");
    }

    #[test]
    fn generate_pseudonymous_de() {
        let mut report_data = example_pseudonymous();
        report_data.report_language = langid!("de");

        assert_snapshot!(generate(&report_data),
        @r"


        OpenTalk Abstimmungsbericht
         Titel : Example Pseudonymous Vote

        Pseudonym : Ja

        Abstimmungsleitung : Alice Adams

        Abstimmungs-ID : 6a3525fc-aeef-4d7e-9d76-e41ab2cbe469

        Beginn : 2025-02-08 12:32:09

        Ende : 2025-02-08 12:32:22

        Zeitzone des Berichts : Europe/Vienna

        Anzahl Teilnehmende : 4

        Geplante Dauer : 60 s

        Enthaltung : zulässig

        Automatisches Ende : aktiviert

        Abstimmung endete auf Grund von : Alle Stimmen abgegeben

        Anzahl abgegebener Stimmen : 4

        Ergebnisse
         Stimme Anzahl

        Zustimmung 1

        Ablehnung 2

        Enthaltung 1

        Recorded votes
         Name Token Stimme Zeitpunkt

        Versteckt LPwNXJWs7b1 Zustimmung —

        Versteckt K5SMSt98f11 Ablehnung —

        Versteckt B1yWM5eWQQi Enthaltung —

        Versteckt 8PCkuJ9NGoY Ablehnung —

        Ereignisprotokoll
         Name Zeitpunkt Ereignis
        ");
    }

    #[test]
    fn generate_canceled_live_roll_call() {
        assert_snapshot!(
            generate(&canceled_public()),
            @r"


        OpenTalk Vote Report
         Title : Weather Vote

        Subtitle : Another one of these weather votes

        Topic : Is the weather good today?

        Pseudonymous : No

        Referendum leader : Alice Adams

        Vote id : ee621ab4-72f6-4d39-bbc4-dc1b96a606cf

        Start : 2025-01-02 03:04:05

        End : 2025-01-02 03:09:05

        Report timezone : Europe/Berlin

        Participant count : 8

        Scheduled duration : 300 s

        Abstention : Disallowed

        Automatic close : Enabled

        Vote ended due to : Aborted for custom reason:   test

        Number of votes : 6

        Recorded votes
         Name Token Vote Timestamp

        Alice Adams aaaaaaaa Approval 2025-01-02 03:04:24

        Bob Burton bbbbbbbb Disapproval 2025-01-02 03:04:20

        Charlie Cooper cccccccc Disapproval 2025-01-02 03:04:21

        Dave Dunn dddddddd Approval 2025-01-02 03:04:19

        Erin Eaton eeeeeeee Approval 2025-01-02 03:06:00

        George Grump gggggggg Approval 2025-01-02 03:06:00

        Event log
         Name Timestamp Event

        Charlie Cooper 2025-01-02 03:04:18 Reports a screenshare issue
        "
        );
    }

    #[test]
    fn generate_canceled_live_roll_call_de() {
        let mut report_data = canceled_public();
        report_data.report_language = langid!("de");

        assert_snapshot!(
            generate(&report_data),
            @r"


        OpenTalk Abstimmungsbericht
         Titel : Weather Vote

        Untertitel : Another one of these weather votes

        Thema : Is the weather good today?

        Pseudonym : Nein

        Abstimmungsleitung : Alice Adams

        Abstimmungs-ID : ee621ab4-72f6-4d39-bbc4-dc1b96a606cf

        Beginn : 2025-01-02 03:04:05

        Ende : 2025-01-02 03:09:05

        Zeitzone des Berichts : Europe/Berlin

        Anzahl Teilnehmende : 8

        Geplante Dauer : 300 s

        Enthaltung : nicht zulässig

        Automatisches Ende : aktiviert

        Abstimmung endete auf Grund von : Abgebrochen mit benutzerdefiniertem Grund:   test

        Anzahl abgegebener Stimmen : 6

        Recorded votes
         Name Token Stimme Zeitpunkt

        Alice Adams aaaaaaaa Zustimmung 2025-01-02 03:04:24

        Bob Burton bbbbbbbb Ablehnung 2025-01-02 03:04:20

        Charlie Cooper cccccccc Ablehnung 2025-01-02 03:04:21

        Dave Dunn dddddddd Zustimmung 2025-01-02 03:04:19

        Erin Eaton eeeeeeee Zustimmung 2025-01-02 03:06:00

        George Grump gggggggg Zustimmung 2025-01-02 03:06:00

        Ereignisprotokoll
         Name Zeitpunkt Ereignis

        Charlie Cooper 2025-01-02 03:04:18 Meldet ein Screenshare-Problem
        "
        );
    }

    #[test]
    fn generate_initiator_left() {
        let mut report_data = initiator_left_public();
        report_data.report_language = langid!("en");

        assert_snapshot!(generate(&report_data));
    }

    #[test]
    fn generate_initiator_left_de() {
        let mut report_data = initiator_left_public();
        report_data.report_language = langid!("de");

        assert_snapshot!(generate(&report_data));
    }

    #[test]
    fn generate_room_destroyed() {
        let mut report_data = room_destroyed_public();
        report_data.report_language = langid!("en");

        assert_snapshot!(generate(&report_data));
    }

    #[test]
    fn generate_room_destroyed_de() {
        let mut report_data = room_destroyed_public();
        report_data.report_language = langid!("de");

        assert_snapshot!(generate(&report_data));
    }
}
