// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_types_common::modules::ModuleId;

/// A trait for defining data for peers sent to the frontend of a signaling module.
pub trait SignalingModulePeerFrontendData:
    serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug
{
    /// The namespace which is used to tag the signaling module participant data
    const NAMESPACE: Option<ModuleId>;
}

impl SignalingModulePeerFrontendData for () {
    const NAMESPACE: Option<ModuleId> = None;
}
