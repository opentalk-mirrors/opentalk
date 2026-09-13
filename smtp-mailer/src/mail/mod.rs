// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{borrow::Cow, collections::HashMap, str::FromStr};

use anyhow::Result;
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use fluent_templates::{FluentLoader, fluent_bundle::FluentValue};
use lettre::{
    Message,
    message::{
        Attachment, Mailbox, MultiPart, SinglePart,
        header::{ContentDisposition, ContentTransferEncoding, ContentType},
    },
};
use opentalk_mail_worker_protocol as proto;
use opentalk_types_common::users::{Language, UserTitle};
use tera::{Kwargs, State, Tera, TeraResult};

use crate::{ics::EventStatus, settings};

mod external_event_cancellation;
mod external_event_update;
mod external_invite;
mod external_uninvite;
mod registered_event_cancellation;
mod registered_event_update;
mod registered_invite;
mod registered_uninvite;
mod unregistered_event_cancellation;
mod unregistered_event_update;
mod unregistered_invite;
mod unregistered_uninvite;

const AVAILABLE_LANGUAGES: &[icu_locid::LanguageIdentifier] =
    &[icu_locid::langid!("en-US"), icu_locid::langid!("de-DE")];

pub(crate) fn create_template_engine(settings: &settings::Settings) -> Result<Tera> {
    let base_path = std::env::var("CARGO_MANIFEST_DIR")
        .map(|p| format!("{p}/"))
        .unwrap_or_default();

    let mut tera = Tera::default();

    // Functions, filters and components are validated at the time a template is added, so
    // everything the templates rely on has to be registered before adding them.
    tera.register_function("fluent", FluentLoader::new(&*crate::i18n::LOCALES));

    tera.register_filter("wrap_text", wrap_text_filter);
    tera.register_filter("build_link", build_link);
    tera.register_filter("space_groups", space_groups_filter);
    tera.register_filter("format_telephone_number", format_telephone_number_filter);
    tera.register_filter("date", tera_contrib::dates::date);

    tera.add_raw_template(
        "macros.html",
        include_str!("../../resources/templates/macros.html"),
    )?;
    tera.add_raw_template(
        "common_styles.css",
        include_str!("../../resources/templates/common_styles.css"),
    )?;
    tera.add_raw_template(
        "header_logo.include",
        include_str!("../../resources/templates/header_logo.include"),
    )?;
    tera.add_raw_template(
        "callin_txt.include",
        include_str!("../../resources/templates/callin_txt.include"),
    )?;
    tera.add_raw_template(
        "callin_html.include",
        include_str!("../../resources/templates/callin_html.include"),
    )?;
    tera.add_raw_template(
        "shared_folder_txt.include",
        include_str!("../../resources/templates/shared_folder_txt.include"),
    )?;
    tera.add_raw_template(
        "shared_folder_html.include",
        include_str!("../../resources/templates/shared_folder_html.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/quick_guide_txt.include"),
        Some("quick_guide_txt.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/quick_guide_html.include"),
        Some("quick_guide_html.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/data_protection_txt.include"),
        Some("data_protection_txt.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/data_protection_html.include"),
        Some("data_protection_html.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/data_protection_ics.include"),
        Some("data_protection_ics.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/adhoc_txt.include"),
        Some("adhoc_txt.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/adhoc_html.include"),
        Some("adhoc_html.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/streaming_links_txt.include"),
        Some("streaming_links_txt.include"),
    )?;
    tera.add_template_file(
        format!("{base_path}resources/templates/streaming_links_html.include"),
        Some("streaming_links_html.include"),
    )?;
    // `ics_description.txt` includes `streaming_links_txt.include` and
    // `data_protection_ics.include`. Includes are validated when the template is added, so it has
    // to come after its dependencies.
    tera.add_raw_template(
        "ics_description.txt",
        include_str!("../../resources/templates/ics_description.txt"),
    )?;
    tera.add_template_files(vec![
        (
            format!("{base_path}resources/templates/external_details_html.include"),
            Some("external_details_html.include"),
        ),
        (
            format!("{base_path}resources/templates/unregistered_details_html.include"),
            Some("unregistered_details_html.include"),
        ),
        (
            format!("{base_path}resources/templates/registered_details_html.include"),
            Some("registered_details_html.include"),
        ),
    ])?;

    tera.add_template_files(
        settings
            .templates
            .iter()
            .map(|(path, name)| (path, Some(name))),
    )?;

    Ok(tera)
}

/// A builder that contains everything needed to render and create a Mail
pub struct MailBuilder {
    frontend: settings::Frontend,
    builder: settings::TemplateBuilder,
    _default_language: Language,
    support_contact: Option<settings::SupportContact>,
    from_name: String,
    from_email: String,
    tera: Tera,
}

/// Trait to render the different tasks
pub trait MailTemplate {
    fn generate_email_plain(&self, builder: &MailBuilder) -> Result<String>;
    fn generate_email_html(&self, builder: &MailBuilder) -> Result<String>;
    fn generate_subject(&self, builder: &MailBuilder) -> Result<String>;
    fn generate_reply_to_mbox(&self, builder: &MailBuilder) -> Result<Mailbox>;
    fn generate_to_mbox(&self, builder: &MailBuilder) -> Result<Mailbox>;
    fn generate_attachments(&self, builder: &MailBuilder) -> Result<Vec<SinglePart>>;
}

impl MailBuilder {
    pub fn new(settings: &settings::Settings) -> Result<Self> {
        let tera = create_template_engine(settings)?;

        Ok(Self {
            frontend: settings.frontend.to_owned(),
            builder: settings.template_builder.to_owned(),
            _default_language: settings.languages.default_language.to_owned(),
            support_contact: settings.support_contact.to_owned(),
            from_name: settings.smtp.from_name.to_owned(),
            from_email: settings.smtp.from_email.to_owned(),
            tera,
        })
    }

    fn create_join_link(&self, event: &proto::v1::Event) -> String {
        let template = &self.builder.join_link_builder;
        // Use the room alias if present, otherwise fall back to the room ID
        let room_alias_or_id = event
            .room
            .alias
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| event.room.id.to_string());

        template
            .replace("{base_url}", &self.frontend.base_url)
            .replace("{room_id}", &room_alias_or_id)
    }

    fn create_dashboard_event_link(&self, event: &proto::v1::Event) -> String {
        let template = &self.builder.dashboard_event_link_builder;
        template
            .replace("{base_url}", &self.frontend.base_url)
            .replace("{event_id}", &event.id.to_string())
    }

    pub(crate) fn generate_email(&self, message: &proto::v1::Message) -> Result<Message> {
        const OPENTALK_LOGO_CONTENT_ID: &str = "opentalk-logo";
        const OPENTALK_LOGO: &[u8] = include_bytes!("../../resources/images/opentalk_logo.png");

        let from_mb = generate_from_mbox(self.from_name.clone(), self.from_email.as_ref())?;

        let reply_to_mb = message.generate_reply_to_mbox(self)?;
        let to_mb = message.generate_to_mbox(self)?;

        let subject = message.generate_subject(self)?;
        let txt = message.generate_email_plain(self)?;
        let html = message.generate_email_html(self)?;

        let mut mail_parts = MultiPart::mixed().multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(txt))
                .multipart(
                    MultiPart::related()
                        .singlepart(SinglePart::html(html))
                        .singlepart(
                            Attachment::new_inline(OPENTALK_LOGO_CONTENT_ID.to_string())
                                .body(OPENTALK_LOGO.to_vec(), mime::IMAGE_PNG.into()),
                        ),
                ),
        );

        for attachment in message.generate_attachments(self)? {
            mail_parts = mail_parts.singlepart(attachment);
        }

        let email = lettre::Message::builder()
            .from(from_mb)
            .reply_to(reply_to_mb)
            .to(to_mb)
            .subject(subject)
            .multipart(mail_parts)?;

        Ok(email)
    }
}

fn generate_mailbox_name(title: &UserTitle, first_name: &str, last_name: &str) -> String {
    format!("{title} {first_name} {last_name}")
        .trim()
        .to_string()
}

macro_rules! forward {
    ($self:ident, $fn:ident, $($arg:ident),*) => {
        match $self {
            // Invites
            ::opentalk_mail_worker_protocol::v1::Message::RegisteredEventInvite(x) => x.$fn($($arg)*),
            ::opentalk_mail_worker_protocol::v1::Message::UnregisteredEventInvite(x) => x.$fn($($arg)*),
            ::opentalk_mail_worker_protocol::v1::Message::ExternalEventInvite(x) => x.$fn($($arg)*),
            // Updates
            ::opentalk_mail_worker_protocol::v1::Message::RegisteredEventUpdate(x) => x.$fn($($arg)*),
            ::opentalk_mail_worker_protocol::v1::Message::UnregisteredEventUpdate(x) => x.$fn($($arg)*),
            ::opentalk_mail_worker_protocol::v1::Message::ExternalEventUpdate(x) => x.$fn($($arg)*),
            // Cancellations
            ::opentalk_mail_worker_protocol::v1::Message::RegisteredEventCancellation(x) => x.$fn($($arg)*),
            ::opentalk_mail_worker_protocol::v1::Message::UnregisteredEventCancellation(x) => x.$fn($($arg)*),
            ::opentalk_mail_worker_protocol::v1::Message::ExternalEventCancellation(x) => x.$fn($($arg)*),
            // Uninvites
            ::opentalk_mail_worker_protocol::v1::Message::RegisteredEventUninvite(x) => x.$fn($($arg)*),
            ::opentalk_mail_worker_protocol::v1::Message::UnregisteredEventUninvite(x) => x.$fn($($arg)*),
            ::opentalk_mail_worker_protocol::v1::Message::ExternalEventUninvite(x) => x.$fn($($arg)*),
        }
    };
}

fn generate_from_mbox(name: String, email: &str) -> anyhow::Result<lettre::message::Mailbox> {
    Ok(Mailbox::new(Some(name), email.parse()?))
}

impl MailTemplate for proto::v1::Message {
    fn generate_email_plain(&self, builder: &MailBuilder) -> Result<String> {
        forward!(self, generate_email_plain, builder)
    }

    fn generate_email_html(&self, builder: &MailBuilder) -> Result<String> {
        forward!(self, generate_email_html, builder)
    }

    fn generate_subject(&self, builder: &MailBuilder) -> Result<String> {
        forward!(self, generate_subject, builder)
    }

    fn generate_reply_to_mbox(&self, builder: &MailBuilder) -> Result<Mailbox> {
        forward!(self, generate_reply_to_mbox, builder)
    }

    fn generate_to_mbox(&self, builder: &MailBuilder) -> Result<Mailbox> {
        forward!(self, generate_to_mbox, builder)
    }

    fn generate_attachments(&self, builder: &MailBuilder) -> Result<Vec<SinglePart>> {
        forward!(self, generate_attachments, builder)
    }
}

pub fn wrap_text_filter(value: &str, kwargs: Kwargs, _: &State) -> TeraResult<String> {
    let width = kwargs.get("width")?.unwrap_or(80);

    let wrapped_string = textwrap::fill(value, width);
    Ok(wrapped_string)
}

pub fn build_link(value: &str, kwargs: Kwargs, _: &State) -> TeraResult<String> {
    let caption = kwargs.get("caption")?.unwrap_or(value);

    let link_string = format!("<a href=\"{value}\">{caption}</a>");
    Ok(link_string)
}

pub fn space_groups_filter(value: &str, _: Kwargs, _: &State) -> TeraResult<String> {
    let grouped_string = value
        .chars()
        .enumerate()
        .fold(String::new(), |mut acc, (index, c)| {
            acc.push(c);
            if index % 4 == 3 {
                acc.push(' ');
            };

            acc
        });

    Ok(grouped_string)
}

pub fn format_telephone_number_filter(
    value: &str,
    kwargs: Kwargs,
    _: &State,
) -> TeraResult<String> {
    let mode = kwargs.get("mode")?.unwrap_or_else(|| "e164".to_owned());
    let mode = match mode.as_str() {
        "international" => phonenumber::Mode::International,
        "national" => phonenumber::Mode::National,
        "e164" => phonenumber::Mode::E164,
        "rfc3966" => phonenumber::Mode::Rfc3966,
        _ => phonenumber::Mode::E164,
    };

    let result = std::panic::catch_unwind(move || phonenumber::parse(None, value));

    let formatted_telephone_number = match result {
        Ok(Ok(number)) => number.format().mode(mode).to_string(),
        e => {
            log::debug!("Failed to parse phone number {e:?}");
            if mode == phonenumber::Mode::Rfc3966 {
                format!("tel:{value}")
            } else {
                value.to_owned()
            }
        }
    };

    Ok(formatted_telephone_number)
}

fn common_subject_args(
    event: &proto::v1::Event,
    inviter: &proto::v1::RegisteredUser,
) -> HashMap<Cow<'static, str>, FluentValue<'static>> {
    let mut args: HashMap<Cow<'_, str>, FluentValue<'static>> = HashMap::new();
    args.insert(
        Cow::from("inviter-first_name"),
        inviter.first_name.clone().into(),
    );
    args.insert(
        Cow::from("inviter-last_name"),
        inviter.last_name.clone().into(),
    );
    args.insert(Cow::from("inviter-title"), inviter.title.to_string().into());
    args.insert(Cow::from("event-name"), event.name.to_string().into());
    args
}

fn registered_invitee_subject_args(
    event: &proto::v1::Event,
    invitee: &proto::v1::RegisteredUser,
    inviter: &proto::v1::RegisteredUser,
) -> HashMap<Cow<'static, str>, FluentValue<'static>> {
    let mut args = common_subject_args(event, inviter);
    args.insert(
        Cow::from("invitee-first_name"),
        invitee.first_name.clone().into(),
    );
    args.insert(
        Cow::from("invitee-last_name"),
        invitee.last_name.clone().into(),
    );
    args.insert(Cow::from("invitee-title"), invitee.title.to_string().into());
    args
}

fn unregistered_invitee_subject_args(
    event: &proto::v1::Event,
    invitee: &proto::v1::UnregisteredUser,
    inviter: &proto::v1::RegisteredUser,
) -> HashMap<Cow<'static, str>, FluentValue<'static>> {
    let mut args = common_subject_args(event, inviter);
    args.insert(
        Cow::from("invitee-first_name"),
        invitee.first_name.clone().into(),
    );
    args.insert(
        Cow::from("invitee-last_name"),
        invitee.last_name.clone().into(),
    );
    args
}

fn external_invitee_subject_args(
    event: &proto::v1::Event,
    inviter: &proto::v1::RegisteredUser,
) -> HashMap<Cow<'static, str>, FluentValue<'static>> {
    common_subject_args(event, inviter)
}

fn create_ics_attachments(ics: Vec<u8>, event_status: EventStatus) -> Vec<SinglePart> {
    let (file_name, content_type) = match event_status {
        EventStatus::Created | EventStatus::Updated => (
            "invite.ics",
            ContentType::parse("text/calendar; charset=utf-8; method=REQUEST;").unwrap(),
        ),

        EventStatus::Cancelled => (
            "cancel.ics",
            ContentType::parse("text/calendar; charset=utf-8; method=CANCEL;").unwrap(),
        ),
    };

    let calendar_attachment = SinglePart::builder()
        .header(content_type)
        .header(ContentTransferEncoding::QuotedPrintable)
        .body(ics.clone());

    let ics_attachment = SinglePart::builder()
        .content_type(ContentType::parse("application/ics").unwrap())
        .header(ContentDisposition::attachment(file_name))
        .header(ContentTransferEncoding::Base64)
        .body(ics);

    vec![calendar_attachment, ics_attachment]
}

pub fn get_fluent_language_identifier(
    language: &Language,
) -> std::result::Result<
    fluent_templates::LanguageIdentifier,
    <fluent_templates::LanguageIdentifier as FromStr>::Err,
> {
    // Return a corresponding unic_langid_impl::LanguageIdentifier (instead of the
    // icu_locid::langid::LanguageIdentifier embedded in
    // opentalk_types_common::users::language::Language)
    language.to_string().parse()
}

fn negotiate_language(language: &Language) -> Language {
    const DEFAULT_LANGUAGE: &icu_locid::LanguageIdentifier = &AVAILABLE_LANGUAGES[0];

    let language_identifier = language
        .to_string()
        .parse()
        .unwrap_or_else(|_| DEFAULT_LANGUAGE.clone());

    let requested_languages: &[_] = &[language_identifier];

    let negotiated_languages = negotiate_languages(
        requested_languages,
        AVAILABLE_LANGUAGES,
        None,
        NegotiationStrategy::Lookup,
    );

    let selected_language = *negotiated_languages.first().unwrap_or(&DEFAULT_LANGUAGE);

    Language(selected_language.clone())
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_mail_worker_protocol::v1::{Event, Room};
    use opentalk_types_common::{rooms::RoomAlias, utils::ExampleData};
    use uuid::Uuid;

    use super::MailBuilder;
    use crate::settings::Settings;

    #[test]
    fn create_join_link_uses_alias_if_present() {
        let builder = MailBuilder::new(&Settings::default()).unwrap();
        let event = Event {
            room: Room {
                id: Uuid::nil(),
                alias: Some(RoomAlias::example_data()),
                password: None,
            },
            ..Event::example_data()
        };

        let produced = builder.create_join_link(&event);
        assert_snapshot!(produced, @"https://opentalk.example.org/room/personal-room-name_0000000000000000");
    }

    #[test]
    fn create_join_link_falls_back_to_room_id() {
        let builder = MailBuilder::new(&Settings::default()).unwrap();
        let event = Event {
            room: Room {
                id: Uuid::nil(),
                alias: None,
                password: None,
            },
            ..Event::example_data()
        };

        let produced = builder.create_join_link(&event);
        assert_snapshot!(produced, @"https://opentalk.example.org/room/00000000-0000-0000-0000-000000000000");
    }
}
