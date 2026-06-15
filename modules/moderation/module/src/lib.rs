// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    mem,
};

use anyhow::anyhow;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_module_shared_folder::{SharedFolderModule, UpdateSharedFolder};
use opentalk_roomserver_signaling::{
    banned_participant::BannedParticipant,
    module_context::{ChannelDroppedError, ModuleContext},
    participant_state::ParticipantState,
    signaling_module::{
        ModuleJoinData, NoOp, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    client_parameters::{ClientKind, Role},
    connection_id::ConnectionId,
    kick_reason::KickReason,
    room_parameters::WaitingRoom,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_livekit::{
    LiveKitInternal, MicrophoneRestrictionError, MicrophoneRestrictionState, ParticipantsMuted,
};
use opentalk_roomserver_types_moderation::{
    KickScope, MODERATION_MODULE_ID,
    command::ModerationCommand,
    event::{BannedParticipantInfo, ModerationError, ModerationEvent},
    state::{
        ChangeDisplayNameRestrictionState, ModerationState, ModeratorJoinInfo,
        WaitingParticipantPeerData,
    },
};
use opentalk_types_common::{modules::ModuleId, users::DisplayName};
use opentalk_types_signaling::ParticipantId;
use tokio::sync::oneshot;

pub struct ModerationModule {
    change_display_name_restriction_state: ChangeDisplayNameRestrictionState,
}

pub enum ModerationLoopback {
    Mute(ParticipantsMuted),
    MicrophoneRestrictionsUpdated(Result<MicrophoneRestrictionState, MicrophoneRestrictionError>),
    ChannelDropped,
}

impl From<ChannelDroppedError> for ModerationLoopback {
    fn from(_: ChannelDroppedError) -> Self {
        Self::ChannelDropped
    }
}

impl SignalingModuleDescription for ModerationModule {
    const MODULE_ID: ModuleId = MODERATION_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles moderation functionality";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];
}

impl SignalingModule for ModerationModule {
    const NAMESPACE: ModuleId = MODERATION_MODULE_ID;

    type Incoming = ModerationCommand;

    type Outgoing = ModerationEvent;

    type Internal = NoOp;

    type Loopback = ModerationLoopback;

    type JoinInfo = ModerationState;

    type PeerJoinInfo = ();

    type Error = ModerationError;

    fn init(_init_data: SignalingModuleInitData) -> Option<Self> {
        Some(Self {
            change_display_name_restriction_state: ChangeDisplayNameRestrictionState::Disabled,
        })
    }

    #[allow(unused_variables)]
    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let moderator_data = ctx
            .is_moderator(participant_id)
            .then(|| Self::moderator_data(ctx));
        let join_info = ModuleJoinData {
            join_success: Some(ModerationState {
                moderator_data,
                display_name_change_restrictions: self
                    .change_display_name_restriction_state
                    .clone(),
            }),
            ..Default::default()
        };

        Ok(join_info)
    }

    #[allow(unused_variables)]
    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        content: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match content {
            ModerationCommand::Kick { target } => Self::kick_participant(ctx, sender, target),
            ModerationCommand::Ban { target } => Self::ban_participant(ctx, sender, target),
            ModerationCommand::Unban { target } => Self::unban_participant(ctx, sender, target),
            ModerationCommand::UpdateRole {
                participant_id,
                new_role,
            } => Self::update_participant_role(ctx, sender, participant_id, new_role),
            ModerationCommand::Debrief(kick_scope) => Self::debrief(ctx, sender, kick_scope),
            ModerationCommand::ChangeWaitingRoomState { new_state: state } => {
                Self::set_waiting_room_state(ctx, sender, state)
            }
            ModerationCommand::Accept { target } => {
                Self::accept_waiting_participant(ctx, sender, target)
            }
            ModerationCommand::SendToWaitingRoom { target } => {
                Self::send_to_waiting_room(ctx, sender, target)
            }
            ModerationCommand::ChangeDisplayName { new_name, target } => {
                self.change_display_name(ctx, sender, new_name, target)
            }
            ModerationCommand::EnableDisplayNameChangeRestrictions {
                unrestricted_participants,
            } => self.enable_display_name_restrictions(ctx, sender, unrestricted_participants),
            ModerationCommand::DisableDisplayNameChangeRestrictions => {
                self.disable_display_name_restrictions(ctx, sender)
            }
            ModerationCommand::Mute { participants } => Self::mute(ctx, sender, participants),
            ModerationCommand::EnableMicrophoneRestrictions {
                unrestricted_participants,
            } => Self::update_microphone_restrictions(
                ctx,
                sender,
                MicrophoneRestrictionState::Enabled {
                    unrestricted_participants,
                },
            ),
            ModerationCommand::DisableMicrophoneRestrictions => {
                Self::update_microphone_restrictions(
                    ctx,
                    sender,
                    MicrophoneRestrictionState::Disabled,
                )
            }
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            ModerationLoopback::Mute(ParticipantsMuted {
                sender,
                participants,
            }) => {
                tracing::debug!(
                    "Following participants were muted by the {} module: {participants:?}",
                    Self::NAMESPACE
                );
                let Some(sender) = sender else {
                    return Err(
                        anyhow!("Mute loopback returned without moderator information").into(),
                    );
                };

                ctx.send_ws_message(participants, ModerationEvent::Muted { moderator: sender })?;
            }
            ModerationLoopback::MicrophoneRestrictionsUpdated(result) => {
                self.notify_microphone_restrictions_updated(ctx, result)?;
            }
            ModerationLoopback::ChannelDropped => {
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).ids(),
                    ModerationEvent::Error(ModerationError::Internal),
                )?;
            }
        }

        Ok(())
    }
}

impl ModerationModule {
    fn kick_participant(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if ctx.is_room_owner(target) {
            return Err(ModerationError::CannotKickRoomOwner.into());
        }

        if ctx.participants.connected().get(&target).is_none() {
            return Err(ModerationError::UnknownParticipant.into());
        }

        // Only the room owner is allowed to kick moderators
        if ctx.is_moderator(target) && !ctx.is_room_owner(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        Self::enable_waiting_room_for_participant(ctx, target)?;

        ctx.send_ws_message(
            [target],
            ModerationEvent::Kicked {
                reason: KickReason::Kicked,
            },
        )?;
        ctx.kick_participants(Vec::from_iter([target]), KickReason::Kicked);

        Ok(())
    }

    fn ban_participant(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if sender == target {
            return Err(ModerationError::CannotBanSelf.into());
        }

        if ctx.is_room_owner(target) {
            return Err(ModerationError::CannotBanRoomOwner.into());
        }

        if ctx.banned_participants.contains_key(&target) {
            return Err(ModerationError::AlreadyBanned.into());
        }

        let user_info = if let Some(participant) = ctx.lobby_participants.get(&target) {
            let user_info = participant
                .kind
                .user_info()
                .cloned()
                .ok_or(ModerationError::CannotBanGuests)?;

            ctx.send_ws_message_to_lobby([target], ModerationEvent::Banned)?;
            ctx.ban_lobby_participant(target);
            user_info
        } else if let Some(target_state) = ctx.participants.all_unfiltered.get(&target) {
            let user_info = target_state
                .kind
                .user_info()
                .cloned()
                .ok_or(ModerationError::CannotBanGuests)?;

            if target_state.is_connected() {
                ctx.send_ws_message([target], ModerationEvent::Banned)?;
            }

            ctx.ban_participant(target);
            user_info
        } else {
            return Err(ModerationError::UnknownParticipant.into());
        };

        let banned_participant = BannedParticipant {
            display_name: user_info.display_name,
            avatar_url: user_info.avatar_url,
            banned_by: sender,
            banned_at: ctx.timestamp,
        };

        // Update ban list
        ctx.banned_participants
            .insert(target, banned_participant.clone());

        let moderators = ctx
            .participants
            .moderators()
            .ids()
            .filter(|participant_id| participant_id != &target);

        ctx.send_ws_message(
            moderators,
            ModerationEvent::ParticipantBanned(BannedParticipantInfo {
                participant_id: target,
                banned_participant,
            }),
        )?;

        Ok(())
    }

    fn unban_participant(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if ctx.banned_participants.remove(&target).is_some() {
            let moderators = ctx.participants.moderators().connected().ids();
            ctx.send_ws_message(
                moderators,
                ModerationEvent::ParticipantUnbanned {
                    participant_id: target,
                },
            )?;
        } else {
            return Err(ModerationError::AlreadyUnbanned.into());
        }

        Ok(())
    }

    fn update_participant_role(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
        new_role: Role,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if ctx.is_room_owner(target) {
            return Err(ModerationError::CannotChangeRoomOwnerRole.into());
        }

        let Some(target_state) = ctx.participants.all_unfiltered.get_mut(&target) else {
            return Err(ModerationError::UnknownParticipant.into());
        };

        if !target_state.kind.is_registered_non_callin_user() {
            return Err(ModerationError::UserCannotBeModerator.into());
        }

        if target_state.role == new_role {
            return Err(ModerationError::RoleAlreadyAssigned.into());
        }

        target_state.role = new_role;

        if new_role.is_moderator() {
            // Send the role update event **with** moderator data to the participant that was
            // promoted to moderator
            ctx.send_ws_message(
                [target],
                ModerationEvent::RoleUpdated {
                    participant_id: target,
                    new_role,
                    moderator_data: Some(Self::moderator_data(ctx)),
                },
            )?;

            // Send the role update event without moderator data to everyone else
            ctx.send_ws_message(
                ctx.participants
                    .connected()
                    .ids()
                    .filter(|id| *id != target),
                ModerationEvent::RoleUpdated {
                    participant_id: target,
                    new_role,
                    moderator_data: None,
                },
            )?;
        } else {
            ctx.send_ws_message(
                ctx.participants.connected().ids(),
                ModerationEvent::RoleUpdated {
                    participant_id: target,
                    new_role,
                    moderator_data: None,
                },
            )?;
        }

        ctx.send_internal_command::<SharedFolderModule>(UpdateSharedFolder {
            participant_id: target,
        });

        Ok(())
    }

    fn debrief(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        scope: KickScope,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        let (kicked, not_kicked) =
            Self::split_by_kick_scope(&ctx.participants.all_unfiltered, scope);

        ctx.send_ws_message(
            not_kicked,
            ModerationEvent::DebriefingStarted { issued_by: sender },
        )?;

        let required_waiting_room = scope.into();
        if required_waiting_room > ctx.room_task_info.room.waiting_room {
            Self::update_waiting_room(ctx, required_waiting_room)?;
        }

        ctx.send_ws_message(
            kicked.clone(),
            ModerationEvent::Kicked {
                reason: KickReason::Debriefed,
            },
        )?;
        ctx.kick_participants(kicked, KickReason::Debriefed);

        Ok(())
    }

    /// Update the waiting room state
    ///
    /// This will accept participants that are allowed to join based on the new state.
    fn set_waiting_room_state(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        state: WaitingRoom,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if ctx.room_task_info.room.waiting_room == state {
            ctx.send_ws_message(
                [sender],
                ModerationEvent::WaitingRoomUpdated { new_state: state },
            )?;
            return Ok(());
        }

        match state {
            WaitingRoom::Disabled => {
                // Waiting room disabled, let everyone in
                // we need to collect here since two references to `ctx` are not allowed
                let waiting_participants: Vec<_> = ctx.lobby_participants.keys().copied().collect();

                for p in waiting_participants {
                    Self::accept_waiting_participant_unchecked(ctx, p)?;
                }
            }
            WaitingRoom::ForGuests => {
                // Waiting room enabled for guests, let invited registered users in
                let waiting_participants: Vec<_> = ctx
                    .lobby_participants
                    .iter()
                    .filter_map(|(id, state)| {
                        if state.kind.is_registered_non_callin_user() {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect();

                for p in waiting_participants {
                    Self::accept_waiting_participant_unchecked(ctx, p)?;
                }
            }
            WaitingRoom::ForEveryone => {}
        }

        Self::update_waiting_room(ctx, state)?;

        Ok(())
    }

    fn split_by_kick_scope(
        participants: &HashMap<ParticipantId, ParticipantState>,
        scope: KickScope,
    ) -> (Vec<ParticipantId>, Vec<ParticipantId>) {
        let mut kicked = Vec::new();
        let mut not_kicked = Vec::new();

        for (id, state) in participants {
            if scope.kicks(state.role, &state.kind) {
                kicked.push(*id);
            } else {
                not_kicked.push(*id);
            }
        }

        (kicked, not_kicked)
    }

    /// Accept the waiting participant and allow them to join the meeting.
    /// Ensures that the sender of the command has moderator permissions.
    fn accept_waiting_participant(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            tracing::debug!("Insufficient permissions");
            return Err(ModerationError::InsufficientPermissions.into());
        }

        Self::accept_waiting_participant_unchecked(ctx, target)
    }

    fn accept_waiting_participant_unchecked(
        ctx: &mut ModuleContext<'_, Self>,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        let Some(participant) = ctx.lobby_participants.get_mut(&target) else {
            tracing::debug!(
                "Failed to send `accept` to waiting participant: participant not known ({target})"
            );
            return Err(ModerationError::NotWaiting.into());
        };

        participant.accepted = true;

        tracing::trace!("accept participant: {target}");
        ctx.send_ws_message_to_lobby([target], ModerationEvent::Accepted)?;

        ctx.send_ws_message(
            ctx.participants.connected().moderators().ids(),
            ModerationEvent::ParticipantAccepted {
                participant_id: target,
            },
        )?;

        Ok(())
    }

    /// Set the waiting room state without checking permissions.
    fn update_waiting_room(
        ctx: &mut ModuleContext<'_, Self>,
        new_state: WaitingRoom,
    ) -> Result<(), FatalError> {
        ctx.room_task_info.room.waiting_room = new_state;
        let event = ModerationEvent::WaitingRoomUpdated { new_state };
        ctx.send_ws_message(ctx.participants.connected().ids(), event)
    }

    fn send_to_waiting_room(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        if ctx.is_room_owner(target) {
            return Err(ModerationError::CannotSendRoomOwnerToWaitingRoom.into());
        }

        Self::enable_waiting_room_for_participant(ctx, target)?;

        ctx.send_ws_message([target], ModerationEvent::SentToWaitingRoom)?;
        ctx.move_to_waiting_room(target);

        Ok(())
    }

    fn change_display_name(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        mut new_name: DisplayName,
        target: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            if target != sender {
                return Err(ModerationError::InsufficientPermissions.into());
            }

            if let ChangeDisplayNameRestrictionState::Enabled {
                unrestricted_participants,
            } = &self.change_display_name_restriction_state
                && !unrestricted_participants.contains(&sender)
            {
                return Err(ModerationError::InsufficientPermissions.into());
            }
        }

        if new_name.as_str().trim().is_empty() || new_name.len() > 100 {
            return Err(ModerationError::InvalidDisplayName.into());
        }
        // Sanitize the display name
        new_name = DisplayName::from_str_lossy(new_name.as_str());

        let Some(participant) = ctx.participants.all_unfiltered.get_mut(&target) else {
            return Err(ModerationError::UnknownParticipant.into());
        };

        let ClientKind::Guest { display_name } = &mut participant.kind else {
            return Err(ModerationError::CannotChangeNameOfRegisteredUsers.into());
        };

        let old_name = mem::replace(display_name, new_name.clone());

        ctx.send_ws_message(
            ctx.participants.connected().ids(),
            ModerationEvent::DisplayNameChanged {
                target,
                issued_by: sender,
                old_name,
                new_name,
            },
        )?;

        Ok(())
    }

    fn enable_display_name_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        unrestricted_participants: HashSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        self.change_display_name_restriction_state = ChangeDisplayNameRestrictionState::Enabled {
            unrestricted_participants: unrestricted_participants.clone(),
        };

        ctx.send_ws_message(
            ctx.participants.connected().ids(),
            ModerationEvent::DisplayNameChangeRestrictionsEnabled {
                unrestricted_participants,
            },
        )?;

        Ok(())
    }

    fn disable_display_name_restrictions(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        self.change_display_name_restriction_state = ChangeDisplayNameRestrictionState::Disabled;

        ctx.send_ws_message(
            ctx.participants.connected().ids(),
            ModerationEvent::DisplayNameChangeRestrictionsDisabled,
        )?;

        Ok(())
    }

    fn mute(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        participants: BTreeSet<ParticipantId>,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        // check that we know all participants and query their connection ids
        let known_participants: BTreeSet<_> = ctx
            .participants
            .connected()
            .ids()
            .filter(|p| participants.contains(p))
            .collect();
        let unknown_participants: BTreeSet<_> = participants
            .difference(&known_participants)
            .copied()
            .collect();

        if !unknown_participants.is_empty() {
            ctx.send_ws_message(
                [sender],
                ModerationEvent::Error(ModerationError::UnknownParticipants {
                    participants: unknown_participants,
                }),
            )?;
        }

        if known_participants.is_empty() {
            // No participants to mute
            return Ok(());
        }

        let (tx, rx) = oneshot::channel();
        ctx.send_internal_command::<LiveKitModule>(LiveKitInternal::Mute {
            sender: Some(sender),
            participants,
            return_channel: tx,
        });

        ctx.recv_loopback(rx, ModerationLoopback::Mute);

        Ok(())
    }

    fn update_microphone_restrictions(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        new_state: MicrophoneRestrictionState,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        let (tx, rx) = oneshot::channel();

        ctx.send_internal_command::<LiveKitModule>(LiveKitInternal::UpdateMicrophoneRestrictions {
            sender,
            new_state,
            return_channel: tx,
        });

        ctx.recv_loopback(rx, ModerationLoopback::MicrophoneRestrictionsUpdated);

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, ctx), fields(room= ?ctx.room))]
    pub fn notify_microphone_restrictions_updated(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        result: Result<MicrophoneRestrictionState, MicrophoneRestrictionError>,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        let state = match result {
            Ok(state) => state,
            Err(err) => {
                ctx.send_ws_message([err.sender], ModerationEvent::Error(err.error.into()))?;
                return Ok(());
            }
        };

        match state {
            MicrophoneRestrictionState::Disabled => {
                ctx.send_ws_message(
                    ctx.participants.connected().room(ctx.room).ids(),
                    ModerationEvent::MicrophoneRestrictionsDisabled,
                )?;
            }
            MicrophoneRestrictionState::Enabled {
                unrestricted_participants,
            } => {
                ctx.send_ws_message(
                    ctx.participants.connected().room(ctx.room).ids(),
                    ModerationEvent::MicrophoneRestrictionsEnabled {
                        unrestricted_participants: unrestricted_participants
                            .iter()
                            .copied()
                            .collect(),
                    },
                )?;
            }
        }

        Ok(())
    }

    /// Enable the most permissive waiting room that still requires the participant to be in the
    /// waiting room.
    fn enable_waiting_room_for_participant(
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        let Some(state) = ctx.participant_state(participant_id) else {
            return Err(ModerationError::UnknownParticipant.into());
        };

        let current_waiting_room = ctx.room_task_info.room.waiting_room;
        let required_waiting_room = current_waiting_room.raise_to_include(&state.kind);
        if required_waiting_room > current_waiting_room {
            Self::update_waiting_room(ctx, required_waiting_room)?;
        }

        Ok(())
    }

    fn moderator_data(ctx: &ModuleContext<'_, Self>) -> ModeratorJoinInfo {
        ModeratorJoinInfo {
            waiting_room: ctx.room_task_info.room.waiting_room,
            guest_access: ctx.room_task_info.room.guest_access,
            waiting_room_participants: ctx
                .lobby_participants
                .iter()
                .map(WaitingParticipantPeerData::from)
                .collect(),
            banned_participants: ctx
                .banned_participants
                .iter()
                .map(BannedParticipantInfo::from)
                .collect(),
        }
    }
}
