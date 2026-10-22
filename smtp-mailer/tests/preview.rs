// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::str::FromStr as _;

use insta::assert_snapshot;
use opentalk_smtp_mailer::{
    preview::{OutputVariant, TemplateVariant},
    settings::Settings,
};
use opentalk_types_common::users::Language;
use rstest::rstest;

fn snapshot_name(template: TemplateVariant, lang: &str, output: OutputVariant) -> String {
    format!("{template:?}-{lang}-{output:?}")
}

#[rstest]
fn preview(
    #[values("de", "en")] lang: &str,
    #[values(OutputVariant::Html, OutputVariant::Plain)] output: OutputVariant,
    #[values(
        TemplateVariant::RegisteredInvite,
        TemplateVariant::RegisteredEventUpdate,
        TemplateVariant::RegisteredCancellation,
        TemplateVariant::RegisteredUninvite,
        TemplateVariant::UnregisteredInvite,
        TemplateVariant::UnregisteredCancellation,
        TemplateVariant::ExternalInvite,
        TemplateVariant::ExternalCancellation
    )]
    template: TemplateVariant,
) {
    let settings = Settings::default();
    let lang = Language::from_str(lang).unwrap();

    let example_mail = template
        .render_template(&settings, output, lang.clone())
        .expect("Rendering must succeed");

    assert_snapshot!(
        snapshot_name(template, &lang.to_string(), output),
        example_mail
    );
}

#[rstest]
#[case("de", "https://example.org/de/quick-guide")]
#[case("en", "https://opentalk.eu/en/quick-manual")]
fn preview_configured_quick_guide_url(
    #[case] lang: &str,
    #[case] expected_url: &str,
    #[values(OutputVariant::Html, OutputVariant::Plain)] output: OutputVariant,
) {
    let mut settings = Settings::default();
    // Only override German, English must fall back to the built-in URL
    settings.frontend.quick_guide_urls = [(
        Language::from_str("de-DE").unwrap(),
        "https://example.org/de/quick-guide".to_owned(),
    )]
    .into();
    let lang = Language::from_str(lang).unwrap();

    let example_mail = TemplateVariant::RegisteredInvite
        .render_template(&settings, output, lang)
        .expect("Rendering must succeed");

    let expected = match output {
        OutputVariant::Html => format!("<a href=\"{expected_url}\">{expected_url}</a>"),
        OutputVariant::Plain => expected_url.to_owned(),
    };
    assert!(
        example_mail.contains(&expected),
        "{expected:?} not found in:\n{example_mail}"
    );
}
