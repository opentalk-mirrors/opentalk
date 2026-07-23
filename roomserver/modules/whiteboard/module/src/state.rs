// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_whiteboard::{
    WHITEBOARD_MODULE_ID, edit_restrictions::EditRestrictions,
};
use opentalk_types_common::modules::ModuleId;
use opentalk_types_signaling::SignalingModuleFrontendData;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhiteboardState {
    pub scene: serde_json::Value,
    pub edit_restrictions: EditRestrictions,
}

impl SignalingModuleFrontendData for WhiteboardState {
    const NAMESPACE: Option<ModuleId> = Some(WHITEBOARD_MODULE_ID);
}
