// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

mod authorization_inventory;
mod types;

pub use authorization_inventory::AuthorizationInventory;
#[cfg(feature = "mockall")]
pub use authorization_inventory::MockAuthorizationInventory;
pub use types::AuthorizationUserRole;
