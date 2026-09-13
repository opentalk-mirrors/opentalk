// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::module_resources::ModuleResourceFilter;

/// The json body for a patch request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct PatchModuleResourceBody {
    /// Filter affected resources
    pub filter: ModuleResourceFilter,
    /// Json patch operations that will be applied to affected resources
    pub patch_operations: Vec<ModuleResourceOperation>,
}

/// json patch operations based on [RFC6902](https://www.rfc-editor.org/rfc/rfc6902#section-4)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum ModuleResourceOperation {
    /// Json patch *add* operation.
    Add {
        /// The path where the value is added.
        path: String,

        /// The value that is added.
        value: serde_json::Value,
    },

    /// Json patch *remove* operation.
    Remove {
        /// The path from where the value is removed.
        path: String,
    },

    /// Json patch *replace* operation.
    Replace {
        /// The path where the value is replaced.
        path: String,

        /// The value by which the existing value is replaced.
        value: serde_json::Value,
    },

    /// Json patch *move* operation.
    Move {
        /// The location from which the value is moved.
        from: String,

        /// The location to which the value is moved.
        path: String,
    },

    /// Json patch *copy* operation.
    Copy {
        /// The location from which the value is copied.
        from: String,

        /// The location to which the value is copied.
        path: String,
    },

    /// Json patch *test* operation.
    Test {
        /// The path of the value that is compared.
        path: String,

        /// The value with which to compare the specified location.
        value: serde_json::Value,
    },
}
