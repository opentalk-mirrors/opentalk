// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::modules::{ModuleId, module_id};

pub const SHARED_FOLDER_MODULE_ID: ModuleId = module_id!("shared_folder");

pub mod command;
pub mod event;
pub mod internal;
pub mod state;
