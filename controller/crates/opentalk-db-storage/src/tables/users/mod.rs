// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains users table structs

mod language_identifier;
mod new_user;
mod serial_user_id;
mod update_user;
mod user;

pub use language_identifier::LanguageIdentifier;
pub use new_user::NewUser;
pub use serial_user_id::SerialUserId;
pub use update_user::UpdateUser;
pub use user::User;
