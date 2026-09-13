// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types for handling modules.

pub use opentalk_types_common_identifiers::module_id::{
    CORE_MODULE_ID, DEFAULT_MODULE_ID, MODULE_ID_MAX_LENGTH, MODULE_ID_MIN_LENGTH,
    MODULE_ID_SCHEMA_CHARS_REGEX, ModuleId, ParseModuleIdError,
};
pub use opentalk_types_common_macros::module_id;
