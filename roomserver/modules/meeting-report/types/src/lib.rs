// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for the OpenTalk meeting-report module.

pub mod command;
pub mod event;

use opentalk_types_common::modules::{ModuleId, module_id};

/// The module id for the signaling module
pub const MEETING_REPORT_MODULE_ID: ModuleId = module_id!("meeting_report");
