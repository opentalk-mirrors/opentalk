// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Types for handling module features.

mod module_feature_id;

pub use module_feature_id::{ModuleFeatureId, ParseModuleFeatureIdError};
pub use opentalk_types_common_identifiers::feature_id::{
    FEATURE_ID_MAX_LENGTH, FEATURE_ID_MIN_LENGTH, FEATURE_ID_SCHEMA_CHARS_REGEX, FeatureId,
    ParseFeatureIdError,
};
use opentalk_types_common_identifiers::module_id::DEFAULT_MODULE_ID;
pub use opentalk_types_common_macros::feature_id;

/// The namespace separator
pub const NAMESPACE_SEPARATOR: &str = "::";

/// The call-in feature identifier string
pub const CALL_IN_FEATURE_ID: FeatureId = feature_id!("call_in");

/// The call-in module feature id
pub const CALL_IN_MODULE_FEATURE_ID: ModuleFeatureId = ModuleFeatureId {
    module: DEFAULT_MODULE_ID,
    feature: CALL_IN_FEATURE_ID,
};

/// The storage-upgradable feature identifier string
pub const STORAGE_UPGRADABLE_FEATURE_ID: FeatureId = feature_id!("storage_upgradable");

/// The storage-upgradable module feature id
pub const STORAGE_UPGRADABLE_MODULE_FEATURE_ID: ModuleFeatureId = ModuleFeatureId {
    module: DEFAULT_MODULE_ID,
    feature: STORAGE_UPGRADABLE_FEATURE_ID,
};

/// The guests_allowed feature identifier string
pub const GUESTS_ALLOWED_FEATURE_ID: FeatureId = feature_id!("guests_allowed");

/// The guests_allowed module feature id
pub const GUESTS_ALLOWED_MODULE_FEATURE_ID: ModuleFeatureId = ModuleFeatureId {
    module: DEFAULT_MODULE_ID,
    feature: GUESTS_ALLOWED_FEATURE_ID,
};
