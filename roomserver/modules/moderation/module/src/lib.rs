// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    mem,
};

use anyhow::{Context as _, anyhow};
use itertools::Itertools as _;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_signaling::{
    banned_participant::BannedParticipant,
    lobby_participant::LobbyParticipant,
    module_context::{ChannelDroppedError, ModuleContext},
    participant_state::ParticipantState,
    signaling_module::{
        ModuleJoinData, NoOp, NoSettings, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
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
use opentalk_types_signaling::{ParticipantId, Role};
use tokio::sync::oneshot;

struct EntryChange {
    now_allowed: Vec<ParticipantId>,
    now_blocked: Vec<ParticipantId>,
    now_accepted: Vec<ParticipantId>,
}

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

    type Settings = NoSettings;
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
        if is_first_connection {
            let display_name = ctx
                .participant_state(participant_id)
                .with_context(|| format!("Participant {participant_id} has no state"))?
                .display_name
                .clone();

            // Notify other connections in the lobby that the display name was assigned
            if ctx.lobby_participants.contains_key(&participant_id) {
                ctx.send_ws_message_to_lobby(
                    [participant_id],
                    ModerationEvent::DisplayNameAssigned {
                        new_name: Some(display_name),
                    },
                )?;
            }
        }

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
        // Check if this is the last connection
        if ctx
            .participant_state(participant_id)
            .is_some_and(|state| state.is_connected())
        {
            return Ok(());
        }

        if let Some(lobby_state) = ctx.lobby_participants.get(&participant_id) {
            // Exclude the disconnecting connection to prevent it from receiving the event when
            // switching to the waiting room
            let connection_ids = lobby_state
                .all_connection_ids()
                .filter(|id| *id != connection_id);
            ctx.send_ws_message_to_lobby_connections(
                connection_ids,
                ModerationEvent::DisplayNameAssigned { new_name: None },
            )?;
        }

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
            ModerationCommand::EnableGuestAccess => Self::enable_guest_access(ctx, sender),
            ModerationCommand::DisableGuestAccess => Self::disable_guest_access(ctx, sender),
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
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    ModerationEvent::Error(ModerationError::Internal),
                )?;
            }
        }

        Ok(())
    }

    fn on_moderation_role_change(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        if let Some(lobby_participant) = ctx.lobby_participants.get(&participant_id) {
            ctx.send_ws_message_to_lobby(
                [participant_id],
                ModerationEvent::EntryPermissionChanged {
                    can_enter: lobby_participant.can_enter(
                        participant_id,
                        &ctx.participants.all_unfiltered,
                        ctx.room_task_info.room.waiting_room,
                    ),
                },
            )?;
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

        // Send the kick event to the targets connection in the conference and the lobby
        ctx.send_ws_message(
            [target],
            ModerationEvent::Kicked {
                reason: KickReason::Kicked,
            },
        )?;
        ctx.send_ws_message_to_lobby(
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

        let user_info = ctx
            .participants
            .all_unfiltered
            .get(&target)
            .map(|state| state.kind.user_info())
            .or_else(|| {
                ctx.lobby_participants
                    .get(&target)
                    .map(|state| state.kind.user_info())
            })
            .ok_or(ModerationError::UnknownParticipant)?
            .ok_or(ModerationError::CannotBanGuests)?
            .clone();

        // Notify all connections of the participant, in the conference and the lobby
        ctx.send_ws_message([target], ModerationEvent::Banned)?;
        ctx.send_ws_message_to_lobby([target], ModerationEvent::Banned)?;

        ctx.ban_participant(target);

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

        if target_state.role.is_guest() || target_state.kind.is_service() {
            return Err(ModerationError::UserCannotBeModerator.into());
        }

        if new_role.is_guest() {
            return Err(ModerationError::UserCannotBeGuest.into());
        }

        if target_state.role == new_role {
            return Err(ModerationError::RoleAlreadyAssigned.into());
        }

        target_state.role = new_role;

        if let Some(lobby_participant) = ctx.lobby_participants.get_mut(&target) {
            lobby_participant.role = new_role;
        }

        if new_role.is_moderator() {
            // Send the role update event **with** moderator data to the participant that was
            // promoted to moderator
            ctx.send_ws_message(
                [target],
                ModerationEvent::RoleUpdated {
                    participant_id: target,
                    new_role,
                    issued_by: sender,
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
                    issued_by: sender,
                    moderator_data: None,
                },
            )?;
        } else {
            ctx.send_ws_message(
                ctx.participants.connected().ids(),
                ModerationEvent::RoleUpdated {
                    participant_id: target,
                    new_role,
                    issued_by: sender,
                    moderator_data: None,
                },
            )?;
        }

        ctx.broadcast_moderation_role_change(target);

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
            not_kicked.iter().copied(),
            ModerationEvent::DebriefingStarted { issued_by: sender },
        )?;

        let required_waiting_room = scope.into();
        if required_waiting_room > ctx.room_task_info.room.waiting_room {
            // Raise the waiting room, but only notify the participants that remain in the
            // conference
            ctx.room_task_info.room.waiting_room = required_waiting_room;
            ctx.send_ws_message(
                not_kicked,
                ModerationEvent::WaitingRoomUpdated {
                    new_state: required_waiting_room,
                },
            )?;
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

    fn enable_guest_access(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        ctx.room_task_info.room.guest_access = true;

        // Notify everyone that guest access was enabled
        ctx.send_ws_message(
            ctx.participants.connected().ids(),
            ModerationEvent::GuestAccessEnabled,
        )?;

        Ok(())
    }

    fn disable_guest_access(
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
    ) -> Result<(), SignalingModuleError<ModerationError>> {
        if !ctx.is_moderator(sender) {
            return Err(ModerationError::InsufficientPermissions.into());
        }

        ctx.room_task_info.room.guest_access = false;

        let mut non_guests = Vec::new();
        let mut guests = Vec::new();
        for (&id, state) in ctx.participants.connected().iter() {
            if state.role.is_guest() {
                guests.push(id);
            } else {
                non_guests.push(id);
            }
        }

        // Notify guests in the conference that they will be kicked
        ctx.send_ws_message(
            guests.clone(),
            ModerationEvent::Kicked {
                reason: KickReason::GuestAccessDisabled,
            },
        )?;

        // Collect guests that are connected to the lobby
        let lobby_guests = ctx
            .lobby_participants
            .iter()
            .filter_map(|(id, state)| state.role.is_guest().then_some(*id));

        // Notify guests in the lobby that they will be kicked
        ctx.send_ws_message_to_lobby(
            lobby_guests.clone(),
            ModerationEvent::Kicked {
                reason: KickReason::GuestAccessDisabled,
            },
        )?;

        // Notify everyone else that guest access was disabled
        ctx.send_ws_message(non_guests, ModerationEvent::GuestAccessDisabled)?;

        // Kick all guests from the conference and the lobby. A guest can be connected to both at
        // the same time, so deduplicate to avoid kicking the same participant twice.
        ctx.kick_participants(
            guests.into_iter().chain(lobby_guests).unique().collect(),
            KickReason::GuestAccessDisabled,
        );

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

        let EntryChange {
            now_accepted,
            now_allowed,
            now_blocked,
        } = Self::partition_lobby_by_entry_change(
            ctx.lobby_participants,
            &ctx.participants.all_unfiltered,
            state,
        );

        ctx.send_ws_message_to_lobby(
            now_allowed,
            ModerationEvent::EntryPermissionChanged { can_enter: true },
        )?;

        ctx.send_ws_message_to_lobby(
            now_blocked,
            ModerationEvent::EntryPermissionChanged { can_enter: false },
        )?;

        for &participant_id in &now_accepted {
            ctx.send_ws_message(
                ctx.participants.moderators().connected().ids(),
                ModerationEvent::ParticipantAccepted { participant_id },
            )?;
        }
        ctx.send_ws_message_to_lobby(now_accepted, ModerationEvent::Accepted)?;

        Self::update_waiting_room(ctx, state)?;

        Ok(())
    }

    fn partition_lobby_by_entry_change(
        lobby_participants: &mut HashMap<ParticipantId, LobbyParticipant>,
        conference_participants: &HashMap<ParticipantId, ParticipantState>,
        waiting_room: WaitingRoom,
    ) -> EntryChange {
        let mut now_allowed = Vec::new();
        let mut now_blocked = Vec::new();
        let mut now_accepted = Vec::new();

        for (&id, participant) in lobby_participants.iter_mut() {
            let allowed = participant.can_enter(id, conference_participants, waiting_room);

            // When the participant is in the waiting room
            if participant.waiting_room_state.is_some() {
                if allowed && !participant.accepted {
                    participant.accepted = true;
                    now_accepted.push(id);
                }
            } else if allowed != participant.accepted {
                participant.accepted = allowed;
                if allowed {
                    now_allowed.push(id);
                } else {
                    now_blocked.push(id);
                }
            }
        }

        EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        }
    }

    fn split_by_kick_scope(
        participants: &HashMap<ParticipantId, ParticipantState>,
        scope: KickScope,
    ) -> (Vec<ParticipantId>, Vec<ParticipantId>) {
        let mut kicked = Vec::new();
        let mut not_kicked = Vec::new();

        for (id, state) in participants {
            if scope.kicks(state.role) {
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

        // Only guests can be renamed, this includes call-in participants
        if !participant.role.is_guest() {
            return Err(ModerationError::CannotChangeNameOfRegisteredUsers.into());
        };

        let old_name = mem::replace(&mut participant.display_name, new_name.clone());
        ctx.send_ws_message(
            ctx.participants.connected().ids(),
            ModerationEvent::DisplayNameChanged {
                target,
                issued_by: sender,
                old_name,
                new_name: new_name.clone(),
            },
        )?;

        // Notify other connections in in the lobby of the affected participant about the name
        // change
        ctx.send_ws_message_to_lobby(
            [target],
            ModerationEvent::DisplayNameAssigned {
                new_name: Some(new_name),
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
        let required_waiting_room = current_waiting_room.raise_to_include(&state.kind, state.role);
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
                .filter_map(|(id, state)| {
                    state.waiting_room_state.as_ref().map(|waiting_room_state| {
                        WaitingParticipantPeerData {
                            participant_id: *id,
                            connections: state.waiting_room_connection_ids().collect(),
                            accepted: state.accepted,
                            joined_at: waiting_room_state.joined_at,
                            display_name: waiting_room_state.display_name.clone(),
                            avatar_url: state.kind.avatar_url().map(String::from),
                        }
                    })
                })
                .collect(),
            banned_participants: ctx
                .banned_participants
                .iter()
                .map(BannedParticipantInfo::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::{
        lobby_participant::{LobbyParticipant, WaitingRoomState},
        participant_state::ParticipantState,
        signaling_module::SignalingModuleDescription as _,
    };
    use opentalk_roomserver_types::{
        client_parameters::ClientKind, public_user_profile::PublicUserProfile,
        room_parameters::WaitingRoom,
    };
    use opentalk_types_common::{time::Timestamp, users::DisplayName, utils::ExampleData};
    use opentalk_types_signaling::{ParticipantId, Role};

    use super::ModerationModule;
    use crate::EntryChange;

    fn lobby_participant(
        kind: ClientKind,
        accepted: bool,
        waiting_room_state: Option<WaitingRoomState>,
    ) -> LobbyParticipant {
        let role = match kind {
            ClientKind::Unregistered | ClientKind::CallIn => Role::Guest,
            _ => Role::User,
        };
        LobbyParticipant::new(kind, role, HashMap::new(), accepted, waiting_room_state)
    }

    fn registered() -> ClientKind {
        ClientKind::Registered {
            profile: PublicUserProfile::example_data(),
        }
    }

    fn waiting_room_state() -> WaitingRoomState {
        WaitingRoomState {
            display_name: DisplayName::from_str_lossy("Alice"),
            joined_at: Timestamp::unix_epoch().into(),
        }
    }

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&ModerationModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[moderation]");
    }

    #[test]
    fn accepted_waiting_participants_never_change() {
        let participant_id = ParticipantId::from_u128(1);
        let mut lobby_participants = HashMap::from_iter([(
            participant_id,
            lobby_participant(ClientKind::Unregistered, true, Some(waiting_room_state())),
        )]);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::Disabled,
        );

        assert!(now_allowed.is_empty());
        assert!(now_blocked.is_empty());
        assert!(now_accepted.is_empty());
        assert!(lobby_participants[&participant_id].accepted);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::ForEveryone,
        );

        assert!(now_allowed.is_empty());
        assert!(now_blocked.is_empty());
        assert!(now_accepted.is_empty());
        assert!(lobby_participants[&participant_id].accepted);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::ForGuests,
        );

        assert!(now_allowed.is_empty());
        assert!(now_blocked.is_empty());
        assert!(now_accepted.is_empty());
        assert!(lobby_participants[&participant_id].accepted);
    }

    #[test]
    fn lobby_guests_gain_entry_when_waiting_room_disabled() {
        let participant_id = ParticipantId::from_u128(1);
        let mut lobby_participants = HashMap::from_iter([(
            participant_id,
            lobby_participant(ClientKind::Unregistered, false, None),
        )]);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::Disabled,
        );

        assert_eq!(now_allowed, vec![participant_id]);
        assert!(now_blocked.is_empty());
        assert!(now_accepted.is_empty());
        assert!(lobby_participants[&participant_id].accepted);
    }

    #[test]
    fn lobby_guests_lose_entry_when_waiting_room_enabled_for_everyone() {
        let participant_id = ParticipantId::from_u128(1);
        let mut lobby_participants = HashMap::from_iter([(
            participant_id,
            lobby_participant(ClientKind::Unregistered, true, None),
        )]);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::ForEveryone,
        );

        assert!(now_allowed.is_empty());
        assert_eq!(now_blocked, vec![participant_id]);
        assert!(now_accepted.is_empty());
        assert!(!lobby_participants[&participant_id].accepted);
    }

    #[test]
    fn lobby_guests_unchanged_when_waiting_room_already_matches_entry() {
        let participant_id = ParticipantId::from_u128(1);

        // Already blocked; a stricter waiting room keeps them blocked.
        let mut lobby_participants = HashMap::from_iter([(
            participant_id,
            lobby_participant(ClientKind::Unregistered, false, None),
        )]);
        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::ForEveryone,
        );
        assert!(now_allowed.is_empty());
        assert!(now_blocked.is_empty());
        assert!(now_accepted.is_empty());
        assert!(!lobby_participants[&participant_id].accepted);
    }

    #[test]
    fn waiting_room_guests_accepted_when_waiting_room_disabled() {
        let participant_id = ParticipantId::from_u128(1);
        let mut lobby_participants = HashMap::from_iter([(
            participant_id,
            lobby_participant(ClientKind::Unregistered, false, Some(waiting_room_state())),
        )]);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::Disabled,
        );

        assert!(now_allowed.is_empty());
        assert!(now_blocked.is_empty());
        assert_eq!(now_accepted, vec![participant_id]);
        assert!(lobby_participants[&participant_id].accepted);
    }

    #[test]
    fn waiting_room_guests_unchanged_when_waiting_room_enabled_for_everyone() {
        let participant_id = ParticipantId::from_u128(1);
        let mut lobby_participants = HashMap::from_iter([(
            participant_id,
            lobby_participant(ClientKind::Unregistered, false, Some(waiting_room_state())),
        )]);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::ForEveryone,
        );

        assert!(now_allowed.is_empty());
        assert!(now_blocked.is_empty());
        assert!(now_accepted.is_empty());
        assert!(!lobby_participants[&participant_id].accepted);
    }

    #[test]
    fn enabling_waiting_room_for_guests_admits_registered_users_and_blocks_guests() {
        let registered_in_lobby = ParticipantId::from_u128(1);
        let guest_in_lobby = ParticipantId::from_u128(2);
        let registered_in_waiting_room = ParticipantId::from_u128(3);
        let guest_in_wating_room = ParticipantId::from_u128(4);

        let mut lobby_participants = HashMap::from_iter([
            (
                registered_in_lobby,
                lobby_participant(registered(), false, None),
            ),
            (
                guest_in_lobby,
                lobby_participant(ClientKind::Unregistered, true, None),
            ),
            (
                registered_in_waiting_room,
                lobby_participant(registered(), false, Some(waiting_room_state())),
            ),
            (
                guest_in_wating_room,
                lobby_participant(ClientKind::Unregistered, false, Some(waiting_room_state())),
            ),
        ]);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &HashMap::new(),
            WaitingRoom::ForGuests,
        );

        assert_eq!(now_allowed, vec![registered_in_lobby]);
        assert_eq!(now_blocked, vec![guest_in_lobby]);
        assert_eq!(now_accepted, vec![registered_in_waiting_room]);

        assert!(lobby_participants[&registered_in_lobby].accepted);
        assert!(!lobby_participants[&guest_in_lobby].accepted);
        assert!(lobby_participants[&registered_in_waiting_room].accepted);
        assert!(!lobby_participants[&guest_in_wating_room].accepted);
    }

    #[test]
    fn lobby_guests_keep_entry_when_already_in_conference() {
        let participant_id = ParticipantId::from_u128(1);
        let mut lobby_participants = HashMap::from_iter([(
            participant_id,
            lobby_participant(ClientKind::Unregistered, true, None),
        )]);

        // The guest already has a conference presence (e.g. reconnecting with another device),
        // so they may re-enter even though `ForEveryone` would otherwise block a guest.
        let conference_participants = HashMap::from_iter([(
            participant_id,
            ParticipantState::new(
                ClientKind::Unregistered,
                DisplayName::from_str_lossy("Alice"),
                Role::Guest,
                Timestamp::unix_epoch().into(),
                false,
            ),
        )]);

        let EntryChange {
            now_allowed,
            now_blocked,
            now_accepted,
        } = ModerationModule::partition_lobby_by_entry_change(
            &mut lobby_participants,
            &conference_participants,
            WaitingRoom::ForEveryone,
        );

        assert!(now_allowed.is_empty());
        assert!(now_blocked.is_empty());
        assert!(now_accepted.is_empty());
        assert!(lobby_participants[&participant_id].accepted);
    }
}
