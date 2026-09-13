// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::modules::ModuleId;

/// A trait for defining data sent to the frontend of a signaling module.
pub trait SignalingModuleFrontendData:
    serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug
{
    /// The namespace which is used to tag the signaling module data
    const NAMESPACE: Option<ModuleId>;
}

impl SignalingModuleFrontendData for () {
    const NAMESPACE: Option<ModuleId> = None;
}

impl SignalingModuleFrontendData for opentalk_types_common::shared_folders::SharedFolder {
    const NAMESPACE: Option<ModuleId> = Some(opentalk_types_common::shared_folders::MODULE_ID);
}
