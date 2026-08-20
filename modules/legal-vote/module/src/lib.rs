// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

//! # Legal vote signaling module
//!
//! This module allows to hold auditable votes. For each vote held, a protocol is created with all
//! relevant events (e.g. start, votes, issues, eligible participants joining or disconnecting,
//! stop). This protocol is then archived as a module resource and can be downloaded as a PDF
//! report. When a vote completes, the results are verified by e.g. checking that the amount of
//! votes does not exceed the amount of eligible participants.
//!
//! ## Functionality
//!
//! When a vote starts, a [`Token`] is generated for each participant allowed to vote and sent
//! exclusively to them. When a participant votes, they must redeem their token. This ensures that
//! only authorized participants can vote and that each participant can only vote once. The tokens
//! are stored on the server side, but are not linked to the [`UserId`] or [`ParticipantId`].
//!
//! Votes can be either `pseudonymous` or public, this is controlled with the
//! [`pseudonymous`](UserParameters::pseudonymous) field in the [`LegalVoteCommand::Start`] command.
//!
//! ### Pseudonymous Votes
//!
//! No participant information is stored in the protocol for any events. When a participants casts a
//! vote, other participants are not notified about it. The result of the votes contain all tokens
//! and the respective [`VoteOption`]. This allows each participant to verify that their vote was
//! counted correctly, but does not allow to link votes to participants.
//!
//! ### Public Votes
//!
//! The voting behavior of the participants is public. For each event that originates from a user,
//! the user info of the respective participant is stored in the protocol. When a participant casts
//! a vote, all other participants receive a websocket message with the current interim result. The
//! result of the vote contains [`ParticipantId`]s/[`UserId`]s and their respective [`VoteOption`].
//!
//! ## Issues
//!
//! During a vote, participants can report issues. These issues are sent to the vote initiator and
//! are included in the protocol.
//!
//! ## Logging
//!
//! The roomserver creates logs using the [`tracing`] crate. These logs include websocket message
//! content on the `debug` or `trace` level. For pseudonymous votes, the roomserver must not be
//! run with `debug` or `trace` level [`tracing`] enabled to ensure votes remain pseudonymous and
//! cannot be correlated with participants.

use std::{
    collections::{
        BTreeMap, HashMap,
        hash_map::{Entry, OccupiedEntry},
    },
    path::PathBuf,
};

use anyhow::{Context, anyhow, bail};
use chrono::Utc;
use opentalk_roomserver_signaling::{
    localization,
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, NoSettings, PeerDataMap, SignalingModule,
        SignalingModuleDescription, SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    client_parameters::ClientKind,
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::{FatalError, SignalingModuleError},
};
use opentalk_roomserver_types_legal_vote::{
    LEGAL_VOTE_MODULE_ID,
    cancel::{CancelReason, CustomCancelReason},
    command::LegalVoteCommand,
    event::{LegalVoteError, LegalVoteEvent, Results, StopKind, VotingRecord},
    issue::Issue,
    parameters::Parameters,
    state::LegalVoteState,
    token::Token,
    user_parameters::UserParameters,
    vote::{LegalVoteId, VoteOption},
};
use opentalk_types_common::{
    modules::ModuleId,
    time::TimeZone,
    users::{DisplayName, UserId},
};
use opentalk_types_signaling::ParticipantId;
use protocol::v1 as proto;
use vote::{ActiveVote, CompletedVote};

use crate::{loopback::LegalVoteLoopback, user_tokens::UserTokens, vote::CanceledVote};

mod loopback;
mod protocol;
mod report;
mod summary;
mod user_tokens;
mod vote;

pub struct LegalVoteModule {
    active_votes: HashMap<RoomKind, ActiveVote>,
    history: HashMap<LegalVoteId, Vec<proto::ProtocolEntry>>,
    typst_package_path: PathBuf,
}

impl SignalingModuleDescription for LegalVoteModule {
    const MODULE_ID: ModuleId = LEGAL_VOTE_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles the legal-vote functionality";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = NoSettings;
}

impl SignalingModule for LegalVoteModule {
    const NAMESPACE: ModuleId = LEGAL_VOTE_MODULE_ID;

    type Incoming = LegalVoteCommand;

    type Outgoing = LegalVoteEvent;

    type Internal = NoOp;

    type Loopback = LegalVoteLoopback;

    type JoinInfo = LegalVoteState;

    type PeerJoinInfo = ();

    type Error = LegalVoteError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let typst_package_path = init_data.settings.reports.typst.packages_path.clone();
        Some(Self {
            active_votes: HashMap::new(),
            history: HashMap::new(),
            typst_package_path,
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let active_vote = self.active_votes.get_mut(&ctx.room);
        let history = self.history.values();
        let summary = summary::from_history(active_vote.as_deref(), history).map_err(FatalError)?;

        if let Some(active_vote) = active_vote
            && let Some(user_id) = ctx.user_id(participant_id)
        {
            active_vote.participant_joined(user_id, participant_id);
        }

        Ok(ModuleJoinData {
            join_success: Some(LegalVoteState { votes: summary }),
            peer_events: PeerDataMap::default(),
            peer_data: PeerDataMap::default(),
        })
    }

    fn on_participant_disconnected(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let Entry::Occupied(mut occupied) = self.active_votes.entry(ctx.room) else {
            return Ok(());
        };

        let Some(user_id) = ctx.user_id(participant_id) else {
            return Ok(());
        };

        let active_vote = occupied.get_mut();
        active_vote.participant_disconnected(user_id, participant_id);

        let was_last_connection = ctx
            .participant_state(participant_id)
            .is_none_or(|state| state.connections.is_empty());
        if was_last_connection && participant_id == occupied.get().parameters().initiator_id {
            // Initiator has left, stop the vote
            let active_vote = occupied.remove();
            self.cancel_vote(
                ctx,
                active_vote,
                user_id,
                participant_id,
                CancelReason::InitiatorLeft,
            )?;
        }

        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        _connection_id: ConnectionId,
        payload: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match payload {
            LegalVoteCommand::Start(user_parameters) => self.start(ctx, sender, user_parameters),
            LegalVoteCommand::Vote {
                legal_vote_id,
                option,
                token,
            } => self.vote(ctx, sender, legal_vote_id, option, token),
            LegalVoteCommand::ReportIssue {
                legal_vote_id,
                issue,
            } => self.report_issue(ctx, sender, legal_vote_id, issue),
            LegalVoteCommand::Stop { legal_vote_id } => self.stop(ctx, sender, legal_vote_id),
            LegalVoteCommand::Cancel {
                legal_vote_id,
                reason,
            } => self.cancel(ctx, sender, legal_vote_id, reason),
            LegalVoteCommand::GeneratePdf {
                legal_vote_id,
                timezone,
            } => self.generate_pdf(ctx, sender, legal_vote_id, timezone),
        }
    }

    fn on_loopback_event(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        event: Self::Loopback,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match event {
            LegalVoteLoopback::ResourceCreated {
                id,
                parameters,
                initiator_user_id,
                initiator_participant_id,
            } => self.start_vote(
                ctx,
                id,
                initiator_user_id,
                initiator_participant_id,
                parameters,
            ),
            LegalVoteLoopback::VoteTimedOut { id } => {
                let Ok(active_vote) = self.active_vote(ctx.room, id).map(OccupiedEntry::remove)
                else {
                    tracing::warn!("Legal vote ended without the timeout being cancelled");
                    return Ok(());
                };
                self.end_vote(ctx, active_vote, StopKind::Expired)
            }
            LegalVoteLoopback::VoteEnded | LegalVoteLoopback::VoteCancelled => Ok(()),
            LegalVoteLoopback::CreatedPdf {
                msg_target,
                legal_vote_id,
                asset,
            } => Ok(ctx.send_ws_message(
                [msg_target],
                LegalVoteEvent::ReportGenerated {
                    filename: asset.filename,
                    legal_vote_id,
                    asset_id: asset.id,
                },
            )?),
            LegalVoteLoopback::ChannelDropped => {
                tracing::error!("Loopback channel dropped, timeout will not complete.");
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    LegalVoteError::Internal.into(),
                )?;
                Ok(())
            }
            LegalVoteLoopback::Error(err) => Err(err),
        }
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        let active_vote = self.active_votes.get(&new_room);
        let history = self.history.values();
        let summary = summary::from_history(active_vote, history).map_err(FatalError)?;
        let state = LegalVoteState { votes: summary };

        let switch_success = ctx
            .participant_state(participant_id)
            .with_context(|| {
                format!("Participant '{participant_id}' switched without participant state")
            })
            .map_err(FatalError)?
            .connections()
            .map(|connection_id| (connection_id, Some(state.clone())))
            .collect();

        let module_switch_data = ModuleSwitchData {
            switch_success,
            peer_events: PeerDataMap::default(),
            peer_data: PeerDataMap::default(),
        };

        // Only registered users are relevant, because others can not vote.
        let Some(user_id) = ctx.user_id(participant_id) else {
            return Ok(module_switch_data);
        };

        // If there is an ongoing vote in the old room
        if let Entry::Occupied(mut occupied) = self.active_votes.entry(old_room) {
            let active_vote = occupied.get_mut();
            active_vote.participant_disconnected(user_id, participant_id);

            if participant_id == active_vote.parameters().initiator_id {
                let active_vote = occupied.remove();
                self.cancel_vote(
                    ctx,
                    active_vote,
                    user_id,
                    participant_id,
                    CancelReason::InitiatorLeft,
                )?;
            }
        }

        // If there is an ongoing vote in the new room
        if let Some(active_vote) = self.active_votes.get_mut(&new_room) {
            active_vote.participant_joined(user_id, participant_id);
        }

        Ok(module_switch_data)
    }

    fn on_breakout_closed(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        let breakout_votes = self
            .active_votes
            .extract_if(|room, _| *room != RoomKind::Main)
            .map(|(_, v)| v)
            .collect();
        self.cancel_votes(ctx, breakout_votes, CancelReason::RoomDestroyed)?;

        Ok(())
    }

    fn on_closing(&mut self, ctx: &mut ModuleContext<'_, Self>) -> Result<(), anyhow::Error> {
        let votes = self.active_votes.drain().map(|(_, v)| v).collect();
        self.cancel_votes(ctx, votes, CancelReason::RoomDestroyed)
            .map_err(|e| anyhow!("Cancelling vote failed: {e:?}"))?;

        Ok(())
    }
}

impl LegalVoteModule {
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn start(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        parameters: UserParameters,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        if !ctx.is_moderator(sender) {
            return Err(LegalVoteError::InsufficientPermissions.into());
        }

        let Some(user_id) = ctx.user_id(sender) else {
            return Err(LegalVoteError::InsufficientPermissions.into());
        };

        // Multiple concurrent votes in the same room are not allowed
        if self.active_votes.contains_key(&ctx.room) {
            return Err(LegalVoteError::VoteAlreadyActive.into());
        }

        let resource_storage = ctx.module_resources();
        ctx.spawn(loopback::create_resource(
            resource_storage,
            user_id,
            sender,
            parameters,
        ));

        Ok(())
    }

    /// Starts the vote by generating [`Token`]s and sending them to the allowed participants.
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn start_vote(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        legal_vote_id: LegalVoteId,
        initiator_user_id: UserId,
        initiator_participant_id: ParticipantId,
        parameters: UserParameters,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        let Entry::Vacant(vacant) = self.active_votes.entry(ctx.room) else {
            return Err(LegalVoteError::VoteAlreadyActive.into());
        };

        // Pseudonymous votes cannot be live
        if parameters.live && parameters.pseudonymous {
            return Err(LegalVoteError::InvalidParameters.into());
        }

        let UserTokens {
            participant_tokens,
            allowed_users,
        } = UserTokens::try_generate(ctx, &parameters.allowed_participants)?;

        let cancel_timeout = parameters.duration.map(|duration| {
            ctx.loopback_after(duration.into(), move || LegalVoteLoopback::VoteTimedOut {
                id: legal_vote_id,
            })
        });

        let parameters = Parameters {
            initiator_id: initiator_participant_id,
            legal_vote_id,
            start_time: Utc::now(),
            max_votes: participant_tokens.len() as u32,
            allowed_users,
            inner: parameters,
            token: None,
        };

        vacant.insert(ActiveVote::new(
            legal_vote_id,
            initiator_user_id,
            participant_tokens.values().copied().collect(),
            parameters.clone(),
            cancel_timeout,
        ));

        for participant_id in ctx.participants.in_room(ctx.room).connected().ids() {
            let parameters = Parameters {
                token: participant_tokens.get(&participant_id).copied(),
                ..parameters.clone()
            };
            ctx.send_ws_message([participant_id], LegalVoteEvent::Started(parameters))?;
        }

        Ok(())
    }

    // Commands are also logged on trace level
    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn vote(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        id: LegalVoteId,
        option: VoteOption,
        token: Token,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        let mut entry = self.active_vote(ctx.room, id)?;
        let active_vote = entry.get_mut();

        active_vote.try_add_vote(ctx, sender, token, option)?;

        // Send a vote success message to the voting participant
        ctx.send_ws_message(
            [sender],
            LegalVoteEvent::Voted {
                legal_vote_id: id,
                vote_option: option,
                issuer: sender,
                consumed_token: token,
            },
        )?;

        // Send vote update when live
        if active_vote.is_live() {
            let voting_record =
                try_into_voting_record(active_vote.protocol()).map_err(FatalError)?;
            tracing::debug!("Sending live vote update: {voting_record:?}");
            ctx.send_ws_message(
                ctx.participants.in_room(ctx.room).connected().ids(),
                LegalVoteEvent::Updated {
                    legal_vote_id: id,
                    results: Results {
                        tally: active_vote.tally(),
                        voting_record,
                    },
                },
            )?;
        }

        // Auto close when all votes are in and auto close is enabled
        if active_vote.should_close() {
            let active_vote = entry.remove();
            self.end_vote(ctx, active_vote, StopKind::Auto)?;
        }

        Ok(())
    }

    #[tracing::instrument(skip_all, fields(id, issue), level = "debug")]
    fn report_issue(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        id: LegalVoteId,
        issue: Issue,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        let mut active_vote = self.active_vote(ctx.room, id)?;
        let active_vote = active_vote.get_mut();
        active_vote.try_report_issue(ctx, sender, issue.clone())?;

        let participant_id = if active_vote.is_hidden() {
            None
        } else {
            Some(sender)
        };
        ctx.send_ws_message(
            [active_vote.parameters().initiator_id, sender],
            LegalVoteEvent::ReportedIssue {
                legal_vote_id: id,
                participant_id,
                issue,
            },
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn stop(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        id: LegalVoteId,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        if !ctx.is_moderator(sender) {
            return Err(LegalVoteError::InsufficientPermissions.into());
        }

        let active_vote = self.active_vote(ctx.room, id)?;
        let active_vote = active_vote.remove();

        self.end_vote(ctx, active_vote, StopKind::ByParticipant(sender))
    }

    /// Stops the active vote, archives the protocol and creates a PDF report (when `create_pdf` is
    /// enabled).
    fn end_vote(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        active_vote: ActiveVote,
        stop_kind: StopKind,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        let protocol_stop_kind = match stop_kind {
            StopKind::ByParticipant(participant_id) => ctx
                .user_id(participant_id)
                .map(proto::StopKind::ByUser)
                .ok_or(LegalVoteError::InsufficientPermissions)?,
            StopKind::Auto => proto::StopKind::Auto,
            StopKind::Expired => proto::StopKind::Expired,
        };

        let legal_vote_id = active_vote.id();
        let CompletedVote {
            parameters,
            end_time,
            protocol,
            results,
        } = active_vote.end(protocol_stop_kind);
        self.history.insert(legal_vote_id, protocol.clone());

        if parameters.inner.create_pdf {
            self.create_pdf(
                ctx,
                legal_vote_id,
                parameters.initiator_id,
                protocol.clone(),
                parameters.inner.timezone,
            )
            .or_else(|e| {
                ctx.send_ws_message(
                    ctx.participants.in_room(ctx.room).connected().ids(),
                    LegalVoteEvent::Error(e),
                )
            })?;
        }

        Self::save_protocol_resource(ctx, legal_vote_id, protocol);

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            LegalVoteEvent::Stopped {
                legal_vote_id,
                results,
                kind: stop_kind,
                end_time,
            },
        )?;

        Ok(())
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn cancel(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        id: LegalVoteId,
        reason: CustomCancelReason,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        if !ctx.is_moderator(sender) {
            return Err(LegalVoteError::InsufficientPermissions.into());
        }

        let Some(user_id) = ctx.user_id(sender) else {
            return Err(LegalVoteError::InsufficientPermissions.into());
        };

        let active_vote = self.active_vote(ctx.room, id)?.remove();
        let reason = CancelReason::from(reason);
        self.cancel_vote(ctx, active_vote, user_id, sender, reason)?;

        Ok(())
    }

    /// Cancels the active vote, archives the protocol and creates a PDF report (when `create_pdf`
    /// is enabled).
    fn cancel_vote(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        active_vote: ActiveVote,
        issuer: UserId,
        msg_target: ParticipantId,
        reason: CancelReason,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        let id = active_vote.id();
        let CanceledVote {
            parameters,
            end_time,
            protocol,
        } = active_vote.cancel(issuer, reason.clone());
        self.history.insert(id, protocol.clone());

        Self::save_protocol_resource(ctx, id, protocol.clone());

        ctx.send_ws_message(
            ctx.participants.in_room(ctx.room).connected().ids(),
            LegalVoteEvent::Canceled {
                legal_vote_id: id,
                reason,
                end_time,
            },
        )?;

        if parameters.inner.create_pdf {
            self.create_pdf(ctx, id, msg_target, protocol, parameters.inner.timezone)?;
        }

        Ok(())
    }

    fn cancel_votes(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        votes: Vec<ActiveVote>,
        reason: CancelReason,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        for active_vote in votes {
            let initiator_participant_id = active_vote.parameters().initiator_id;
            let initiator_user_id = ctx
                .user_id(initiator_participant_id)
                .context("Could not find user ID of vote initiator")
                .map_err(FatalError)?;
            self.cancel_vote(
                ctx,
                active_vote,
                // TODO: The user id is necessary because it is stored in the protocol. Ideally
                // this would not be the case when a vote is cancelled with reason RoomDestroyed,
                // but this requires a breaking change in the protocol.
                initiator_user_id,
                initiator_participant_id,
                reason.clone(),
            )?;
        }

        Ok(())
    }

    fn save_protocol_resource(
        ctx: &mut ModuleContext<'_, LegalVoteModule>,
        legal_vote_id: LegalVoteId,
        protocol: Vec<proto::ProtocolEntry>,
    ) {
        let module_resources = ctx.module_resources();
        ctx.spawn_optional(async move {
            match loopback::upload_results(module_resources, legal_vote_id, protocol).await {
                Ok(()) => None,
                Err(err) => Some(err.into()),
            }
        });
    }

    #[tracing::instrument(skip(self, ctx), level = "debug")]
    fn generate_pdf(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        id: LegalVoteId,
        time_zone: Option<TimeZone>,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        if !ctx.is_moderator(sender) {
            return Err(LegalVoteError::InsufficientPermissions.into());
        }

        let Some(protocol) = self.history.get(&id).cloned() else {
            return Err(LegalVoteError::InvalidVoteId.into());
        };

        self.create_pdf(ctx, id, sender, protocol, time_zone)?;

        Ok(())
    }

    fn create_pdf(
        &self,
        ctx: &mut ModuleContext<'_, Self>,
        legal_vote_id: LegalVoteId,
        msg_target: ParticipantId,
        protocol: Vec<proto::ProtocolEntry>,
        explicit_time_zone: Option<TimeZone>,
    ) -> Result<(), LegalVoteError> {
        // Fall back to the following time zones in order:
        // 1. `explicit_time_zone` (from parameters or the `GeneratePdf` command)
        // 2. Time zone of the participant (command sender or vote issuer)
        // 3. UTC
        let time_zone = explicit_time_zone.unwrap_or_else(|| {
            ctx.participant_state(msg_target)
                .and_then(|state| state.kind.time_zone())
                .unwrap_or_else(TimeZone::utc)
        });

        let user_names = ctx
            .participants
            .all_unfiltered
            .values()
            .filter_map(|state| match &state.kind {
                ClientKind::Registered { profile } => Some((
                    profile.id,
                    DisplayName::from_str_lossy(&format!(
                        "{} {}",
                        profile.user_info.firstname, profile.user_info.lastname
                    )),
                )),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();

        let report_language = localization::negotiate_languages(ctx, report::AVAILABLE_LANGUAGES)
            .ok_or(LegalVoteError::GenerateReport)?;
        let typst_package_path = self.typst_package_path.clone();
        ctx.spawn(loopback::generate_pdf(
            ctx.assets(),
            legal_vote_id,
            msg_target,
            time_zone,
            ctx.timestamp,
            protocol,
            user_names,
            report_language,
            typst_package_path,
        ));

        Ok(())
    }

    /// Get the active vote in a specific `room` with a specified `id`.
    ///
    /// # Errors
    ///
    /// - [`LegalVoteError::InvalidVoteId`] if the active vote in the specified room does not match
    ///   the provided id.
    /// - [`LegalVoteError::NoVoteActive`] if there is no active vote in the specified room.
    fn active_vote(
        &mut self,
        room: RoomKind,
        id: LegalVoteId,
    ) -> Result<OccupiedEntry<'_, RoomKind, ActiveVote>, SignalingModuleError<LegalVoteError>> {
        match self.active_votes.entry(room) {
            Entry::Occupied(occupied) if occupied.get().id() == id => Ok(occupied),
            Entry::Occupied(..) => Err(LegalVoteError::InvalidVoteId.into()),
            Entry::Vacant(..) => Err(LegalVoteError::NoVoteActive.into()),
        }
    }
}

/// Tries to transform a protocol to a [`VotingRecord`] for inclusion in the vote results.
///
/// # Errors
///
/// [`anyhow::Error`]: if the protocol is inconsistent.
fn try_into_voting_record(protocol: &[proto::ProtocolEntry]) -> anyhow::Result<VotingRecord> {
    let pseudonymous = protocol
        .iter()
        .find_map(|entry| match &entry.event {
            proto::VoteEvent::Start { parameters, .. } => Some(parameters.inner.pseudonymous),
            _ => None,
        })
        .context("Missing `Start` entry in the legal vote protocol.")?;

    let vote_iter = protocol.iter().filter_map(|entry| match &entry.event {
        proto::VoteEvent::Vote {
            user_info,
            token,
            option,
        } => Some((user_info, token, option)),
        _ => None,
    });

    if pseudonymous {
        let tokens = vote_iter
            .map(|(user_info, &token, &option)| {
                if user_info.is_some() {
                    bail!("Protocol contains inconsistent entries.");
                }
                Ok((token, option))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(VotingRecord::TokenVotes(tokens))
    } else {
        let voters = vote_iter
            .map(|(user_info, _, &option)| {
                let user_info = user_info.context("Protocol contains inconsistent entries.")?;
                Ok((user_info.participant_id, option))
            })
            .collect::<anyhow::Result<_>>()?;
        Ok(VotingRecord::UserVotes(voters))
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::LegalVoteModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&LegalVoteModule::example_config()).unwrap();

        assert_snapshot!(produced, @"[legal_vote]");
    }
}
