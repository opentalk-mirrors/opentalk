// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{cell::RefCell, collections::HashMap, sync::Arc};

use futures::stream::FuturesUnordered;
use opentalk_roomserver_signaling::{
    banned_participant::BannedParticipant,
    event_origin::EventOrigin,
    lobby_participant::LobbyParticipant,
    loopback::LoopbackFuture,
    module_context::{ModuleContext, ModuleMessage},
    participant_state::Participants,
    room_info::RoomTaskInfo,
    signaling_module::SignalingModule,
    storage::{
        assets::provider::AssetStorageProvider, module_resources::provider::ModuleResourceProvider,
    },
};
use opentalk_roomserver_types::room_kind::RoomKind;
use opentalk_types_common::{rooms::RoomId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;

/// Contains the state of the [`RoomTask`](super::super::task::RoomTask) that is accessible to all
/// [`SignalingModule`]s
pub struct DynModuleContext<'ctx> {
    pub room_id: RoomId,
    pub room: RoomKind,
    pub event_origin: EventOrigin,
    pub room_task_info: &'ctx mut RoomTaskInfo,
    pub participants: &'ctx mut Participants,
    pub lobby_participants: &'ctx mut HashMap<ParticipantId, LobbyParticipant>,
    pub banned_participants: &'ctx mut HashMap<ParticipantId, BannedParticipant>,
    pub timestamp: Timestamp,
    pub assets: Arc<dyn AssetStorageProvider>,
    pub module_resources: Arc<dyn ModuleResourceProvider>,
    pub messages: &'ctx mut RefCell<Vec<ModuleMessage>>,
    loopback_futures: &'ctx mut FuturesUnordered<LoopbackFuture>,
}

impl<'ctx> DynModuleContext<'ctx> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        room_id: RoomId,
        room: RoomKind,
        event_origin: EventOrigin,
        room_task_info: &'ctx mut RoomTaskInfo,
        participants: &'ctx mut Participants,
        lobby_participants: &'ctx mut HashMap<ParticipantId, LobbyParticipant>,
        banned_participants: &'ctx mut HashMap<ParticipantId, BannedParticipant>,
        timestamp: Timestamp,
        assets: Arc<dyn AssetStorageProvider>,
        module_resources: Arc<dyn ModuleResourceProvider>,
        messages: &'ctx mut RefCell<Vec<ModuleMessage>>,
        loopback_futures: &'ctx mut FuturesUnordered<LoopbackFuture>,
    ) -> Self {
        Self {
            room_id,
            room,
            event_origin,
            room_task_info,
            participants,
            lobby_participants,
            banned_participants,
            timestamp,
            assets,
            module_resources,
            messages,
            loopback_futures,
        }
    }

    /// Create a owned copy of self with a narrower lifetime
    pub(crate) fn reborrow(&mut self) -> DynModuleContext<'_> {
        DynModuleContext {
            room_id: self.room_id,
            room: self.room,
            event_origin: self.event_origin,
            room_task_info: self.room_task_info,
            participants: self.participants,
            lobby_participants: self.lobby_participants,
            banned_participants: self.banned_participants,
            timestamp: self.timestamp,
            assets: Arc::clone(&self.assets),
            module_resources: Arc::clone(&self.module_resources),
            messages: self.messages,
            loopback_futures: self.loopback_futures,
        }
    }

    pub(crate) fn into_typed_context<M: SignalingModule + 'static>(self) -> ModuleContext<'ctx, M> {
        ModuleContext::new(
            self.room_id,
            self.room,
            self.event_origin,
            self.room_task_info,
            self.messages,
            self.participants,
            self.lobby_participants,
            self.banned_participants,
            self.timestamp,
            self.loopback_futures,
            self.assets,
            self.module_resources,
        )
    }
}
