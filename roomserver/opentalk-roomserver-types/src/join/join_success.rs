// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, BTreeSet};

use opentalk_types_common::{
    events::MeetingDetails, features::FeatureId, modules::ModuleId, time::Timestamp,
    users::DisplayName,
};
use opentalk_types_signaling::Role;
use serde::{Deserialize, Serialize};

use crate::{
    client_parameters::ParticipationKind,
    join::{connection_info::ConnectionInfo, event_info::EventInfo, participant::Participant},
    room_info::RoomInfo,
    tariff_details::TariffDetails,
};

/// The data received by a participant upon successfully joining a meeting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinSuccess {
    /// The other active connections and devices of the participant who joined
    pub connections: Vec<ConnectionInfo>,

    /// The display name of the participant who joined
    pub display_name: DisplayName,

    /// The URL to the avatar of the participant who joined
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    /// The role of the participant in the meeting
    pub role: Role,

    /// The type of participant and how they connected to the meeting
    pub participation_kind: ParticipationKind,

    /// The timestamp when the meeting will close
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closes_at: Option<Timestamp>,

    /// The tariff of the meeting
    pub tariff: Box<TariffDetails>,

    /// The map of enabled modules and their available features in the room
    pub enabled_modules: BTreeMap<ModuleId, BTreeSet<FeatureId>>,

    /// The module data for the participant
    pub module_data: opentalk_types_signaling::ModuleData,

    /// List of participants in the meeting
    pub participants: Vec<Participant>,

    /// Information about the event which is associated with the room
    #[serde(default)]
    pub event_info: Option<EventInfo>,

    /// Information about the meeting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meeting_details: Option<MeetingDetails>,

    /// Information about the current room
    pub room_info: RoomInfo,

    /// Flag indicating if the participant is the room owner
    #[serde(default)]
    pub is_room_owner: bool,
}

impl JoinSuccess {
    /// Gets the inner module of a JoinSuccess Message
    pub fn get_module<T: opentalk_types_signaling::SignalingModuleFrontendData>(
        &self,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.get()
    }
}
