// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    any::type_name,
    collections::{BTreeMap, BTreeSet},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use opentalk_roomserver_types::{
    breakout::module_data::BreakoutPeerModuleData, connection_id::ConnectionId,
    core::state::CoreState, room_kind::RoomKind, shared_json::SharedJson,
};
use opentalk_roomserver_types_raise_hands::state::RaisedHandPeerState;
use opentalk_roomserver_types_recording::peer_state::RecordingPeerState;
use serde::Deserialize;
use types::{
    common::{modules::ModuleId, users::DisplayName},
    signaling::{ParticipantId, SignalingModulePeerFrontendData},
};

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: ParticipantId,
    pub room: RoomKind,
    pub hand_is_up: bool,
    pub display_name: DisplayName,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub consents_recording: bool,
    pub connections: BTreeSet<ConnectionId>,
}

impl Participant {
    pub(crate) fn new(
        participant_id: ParticipantId,
        connection_ids: impl IntoIterator<Item = ConnectionId>,
        peer_module_data: &BTreeMap<ModuleId, SharedJson>,
    ) -> Result<Participant> {
        let core_state: CoreState = get_module(peer_module_data)?.context("Missing core module")?;
        let breakout_data: BreakoutPeerModuleData =
            get_module(peer_module_data)?.context("Missing breakout module")?;
        let raise_hands_state: Option<RaisedHandPeerState> = get_module(peer_module_data)?;
        let consents_recording = if let Ok(Some(recording_peer_state)) =
            get_module::<RecordingPeerState>(peer_module_data)
        {
            recording_peer_state.consents_recording
        } else {
            false
        };

        Ok(Participant {
            id: participant_id,
            room: breakout_data.room,
            hand_is_up: raise_hands_state.is_some(),
            display_name: core_state.display_name,
            joined_at: core_state.joined_at.into(),
            left_at: core_state.left_at.map(Into::into),
            consents_recording,
            connections: BTreeSet::from_iter(connection_ids),
        })
    }

    pub(crate) fn handle_breakout_switch(
        &mut self,
        new_room: RoomKind,
        module_data: &BTreeMap<ModuleId, SharedJson>,
    ) -> Result<()> {
        self.room = new_room;

        let consents_recording =
            if let Ok(Some(recording_peer_state)) = get_module::<RecordingPeerState>(module_data) {
                recording_peer_state.consents_recording
            } else {
                false
            };

        self.consents_recording = consents_recording;

        let raise_hands_state = get_module::<RaisedHandPeerState>(module_data)?;
        self.hand_is_up = raise_hands_state.is_some();

        Ok(())
    }
}

fn get_module<T: SignalingModulePeerFrontendData>(
    module_data: &BTreeMap<ModuleId, SharedJson>,
) -> Result<Option<T>> {
    module_data
        .get(&T::NAMESPACE.unwrap())
        .map(parse_module)
        .transpose()
}

fn parse_module<T: for<'a> Deserialize<'a>>(json: &SharedJson) -> Result<T> {
    serde_json::from_value(json.clone_inner()).with_context(|| {
        format!(
            "Failed to parse {} from participant peer data",
            type_name::<T>()
        )
    })
}
