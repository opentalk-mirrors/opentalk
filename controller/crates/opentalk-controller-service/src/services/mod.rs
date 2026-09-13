// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Long Running Services that expose clean APIs and hide implementation details from endpoints
//! If the amount of services grow, add another layer that bundles all services.
mod mail;

pub use mail::{
    ExternalMailRecipient, MailRecipient, MailService, RegisteredMailRecipient,
    UnregisteredMailRecipient,
};
