// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{borrow::Cow, collections::HashMap};

use fluent_templates::{Loader, fluent_bundle::FluentValue};
use lettre::message::{Mailbox, SinglePart};
use opentalk_mail_worker_protocol::{self as protocol, v1::UnregisteredEventUninvite};
use opentalk_types_common::users::{Language, UserTitle};

use super::{
    MailTemplate, create_ics_attachments, generate_mailbox_name, get_fluent_language_identifier,
    negotiate_language,
};
use crate::{
    i18n,
    ics::{EventStatus, create_ics_v1},
};

fn language(obj: &UnregisteredEventUninvite) -> &Language {
    &obj.inviter.language
}

fn build_template_context(
    obj: &UnregisteredEventUninvite,
    builder: &super::MailBuilder,
) -> tera::Context {
    let mut context = tera::Context::new();

    let language = language(obj);
    let language = negotiate_language(language);
    context.insert("language", &language);
    context.insert("invitee", &obj.invitee);
    context.insert("inviter", &obj.inviter);
    context.insert("event", &obj.event);
    context.insert("join_link", &builder.create_join_link(&obj.event));
    context.insert(
        "event_link",
        &builder.create_dashboard_event_link(&obj.event),
    );
    context.insert("support", &builder.support_contact);
    context.insert("data_protection_url", &builder.frontend.data_protection_url);
    context
}

impl MailTemplate for UnregisteredEventUninvite {
    fn generate_email_plain(&self, builder: &super::MailBuilder) -> anyhow::Result<String> {
        let context = build_template_context(self, builder);

        builder
            .tera
            .render("unregistered_uninvite.txt", &context)
            .map_err(Into::into)
    }

    fn generate_email_html(&self, builder: &super::MailBuilder) -> anyhow::Result<String> {
        let context = build_template_context(self, builder);

        let html = builder
            .tera
            .render("unregistered_uninvite.html", &context)?;

        let inliner = css_inline::CSSInliner::options().build();
        inliner.inline(&html).map_err(Into::into)
    }

    fn generate_subject(&self, _builder: &super::MailBuilder) -> anyhow::Result<String> {
        let subject_args = subject_args(&self.event, &self.invitee, &self.inviter);

        let language = language(self);
        let language = negotiate_language(language);
        let lang = get_fluent_language_identifier(&language)?;

        Ok(i18n::LOCALES.lookup_complete(&lang, "event-uninvite-subject", Some(&subject_args)))
    }

    fn generate_reply_to_mbox(
        &self,
        _builder: &super::MailBuilder,
    ) -> anyhow::Result<lettre::message::Mailbox> {
        let mbox = Mailbox::new(
            Some(generate_mailbox_name(
                &self.inviter.title,
                &self.inviter.first_name,
                &self.inviter.last_name,
            )),
            self.inviter.email.as_ref().parse()?,
        );

        Ok(mbox)
    }

    fn generate_to_mbox(&self, _builder: &super::MailBuilder) -> anyhow::Result<Mailbox> {
        let mbox = Mailbox::new(
            Some(generate_mailbox_name(
                &UserTitle::new(),
                &self.invitee.first_name,
                &self.invitee.last_name,
            )),
            self.invitee.email.as_ref().parse()?,
        );

        Ok(mbox)
    }

    fn generate_attachments(
        &self,
        builder: &crate::MailBuilder,
    ) -> anyhow::Result<Vec<SinglePart>> {
        let language = &self.inviter.language;
        let language = negotiate_language(language);

        let mut context = tera::Context::new();
        context.insert("meeting_link", &builder.create_join_link(&self.event));
        context.insert("language", &language);
        context.insert("event", &self.event);
        context.insert("data_protection_url", &builder.frontend.data_protection_url);

        let description = builder.tera.render("ics_description.txt", &context)?;

        let name = format!("{} {}", self.invitee.first_name, self.invitee.last_name);
        let invitee = crate::ics::Invitee::WithName {
            email: self.invitee.email.as_ref(),
            name: &name,
        };

        let ics = create_ics_v1(
            &self.inviter,
            &self.event,
            None,
            invitee,
            &description,
            EventStatus::Cancelled,
        )?;

        if let Some(ics) = ics {
            return Ok(create_ics_attachments(ics, EventStatus::Cancelled));
        }

        Ok(vec![])
    }
}

fn subject_args(
    event: &protocol::v1::Event,
    invitee: &protocol::v1::UnregisteredUser,
    inviter: &protocol::v1::RegisteredUser,
) -> HashMap<Cow<'static, str>, FluentValue<'static>> {
    super::unregistered_invitee_subject_args(event, invitee, inviter)
}
