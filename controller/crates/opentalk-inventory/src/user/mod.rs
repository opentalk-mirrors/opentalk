// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::module_inception)]

mod new_user;
mod update_user;
mod user;
mod user_inventory;

pub use new_user::NewUser;
pub use update_user::UpdateUser;
pub use user::User;
#[cfg(feature = "mockall")]
pub use user_inventory::MockUserInventory;
pub use user_inventory::UserInventory;
