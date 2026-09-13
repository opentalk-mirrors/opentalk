// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::Serialize;

#[derive(
    Debug, PartialEq, Serialize, strum::Display, strum::FromRepr, strum::AsRefStr, strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
pub enum JsonPatchErrorCode {
    /// The given path is invalid
    #[strum(serialize = "ot_invalid_path", to_string = "invalid_path")]
    InvalidPath,

    /// Can only be thrown by the
    /// [`Test`](opentalk_inventory::ModuleResourceOperation::Test) when the
    /// compare returns `false`
    #[strum(serialize = "ot_value_not_equal", to_string = "value_not_equal")]
    ValueNotEqual,

    /// Can only be thrown by a
    /// [`Copy`](opentalk_inventory::ModuleResourceOperation::Copy) or
    /// [`Move`](opentalk_inventory::ModuleResourceOperation::Move) operation
    /// when the `from` parameter is invalid.
    #[strum(serialize = "ot_invalid_from_path", to_string = "invalid_from_path")]
    InvalidFromPath,
}
