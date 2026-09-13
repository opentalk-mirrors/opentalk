// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::HashSet;

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use opentalk_roomserver_signaling::module_context::ModuleContext;
use opentalk_roomserver_types::signaling::module_error::SignalingModuleError;
use opentalk_roomserver_types_legal_vote::{
    cancel::CancelReason,
    event::{self, LegalVoteError, Results},
    invalid::Invalid,
    issue::Issue,
    parameters::Parameters,
    tally::Tally,
    token::Token,
    vote::{LegalVoteId, VoteOption},
};
use opentalk_types_common::users::UserId;
use opentalk_types_signaling::ParticipantId;
use tokio::sync::oneshot::Sender;

use crate::{
    LegalVoteModule,
    loopback::LegalVoteLoopback,
    protocol::v1::{FinalResults, ProtocolEntry, StopKind, UserInfo, VoteEvent},
    try_into_voting_record,
};

/// An abstraction over the voting process and protocol. This struct ensures that only valid actions
/// are performed and if so, they are recorded in the protocol.
pub struct ActiveVote {
    id: LegalVoteId,
    allowed_tokens: HashSet<Token>,
    parameters: Parameters,
    tally: Tally,
    protocol: Vec<ProtocolEntry>,
    timeout_cancel: Option<Sender<LegalVoteLoopback>>,
}

/// A vote that has been completed and does not provide any mutating methods anymore.
pub struct CompletedVote {
    pub parameters: Parameters,
    pub end_time: DateTime<Utc>,
    pub protocol: Vec<ProtocolEntry>,
    pub results: event::FinalResults,
}

/// A vote that has been canceled and does not provide any mutating methods anymore.
pub struct CanceledVote {
    pub parameters: Parameters,
    pub end_time: DateTime<Utc>,
    pub protocol: Vec<ProtocolEntry>,
}

impl ActiveVote {
    pub fn new(
        id: LegalVoteId,
        issuer: UserId,
        allowed_tokens: HashSet<Token>,
        parameters: Parameters,
        cancel_timeout: Option<Sender<LegalVoteLoopback>>,
    ) -> Self {
        let start_entry = ProtocolEntry::new_with_time(
            Utc::now(),
            VoteEvent::Start {
                issuer,
                parameters: Box::new(parameters.clone()),
            },
        );
        let tally = Tally {
            yes: 0,
            no: 0,
            abstain: parameters.inner.enable_abstain.then_some(0),
        };
        Self {
            id,
            allowed_tokens,
            parameters,
            tally,
            protocol: vec![start_entry],
            timeout_cancel: cancel_timeout,
        }
    }

    /// Try to add a vote to the protocol.
    ///
    /// # Errors
    ///
    /// - [`LegalVoteError::InvalidOption`] - The provided option is invalid (e.g. abstain is
    ///   disabled).
    /// - [`LegalVoteError::Ineligible`] - The provided token is not valid or has already been used.
    /// - [`SignalingModuleError::Internal`] - The participant does not have a state.
    pub fn try_add_vote(
        &mut self,
        ctx: &mut ModuleContext<'_, LegalVoteModule>,
        participant_id: ParticipantId,
        token: Token,
        option: VoteOption,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        if option == VoteOption::Abstain && !self.parameters.inner.enable_abstain {
            return Err(LegalVoteError::InvalidOption.into());
        }

        // Consume the token to prevent user from voting multiple times
        if !self.allowed_tokens.remove(&token) {
            return Err(LegalVoteError::InvalidToken.into());
        }

        let (user_info, timestamp) = if self.is_hidden() {
            (None, None)
        } else {
            let user_info = UserInfo {
                issuer: ctx.user_id(participant_id).with_context(|| {
                    format!("Participant '{participant_id}' does not have a state")
                })?,
                participant_id,
            };
            (Some(user_info), Some(Utc::now()))
        };
        let entry = ProtocolEntry::new_with_optional_time(
            timestamp,
            VoteEvent::Vote {
                user_info,
                token,
                option,
            },
        );
        self.protocol.push(entry);

        match option {
            VoteOption::Yes => self.tally.yes += 1,
            VoteOption::No => self.tally.no += 1,
            VoteOption::Abstain => *self.tally.abstain.get_or_insert(0) += 1,
        }

        Ok(())
    }

    /// Try to add an issue to the protocol.
    ///
    /// # Errors
    ///
    /// - [`LegalVoteError::InsufficientPermissions`] - The participant does not have permissions to
    ///   take part in the vote and thus cannot report issues.
    pub fn try_report_issue(
        &mut self,
        ctx: &mut ModuleContext<'_, LegalVoteModule>,
        participant_id: ParticipantId,
        issue: Issue,
    ) -> Result<(), SignalingModuleError<LegalVoteError>> {
        let user_id = ctx
            .participant_state(participant_id)
            .and_then(|state| state.kind.user_id())
            .ok_or(LegalVoteError::InsufficientPermissions)?;

        if !self.parameters.allowed_users.contains(&user_id) {
            return Err(LegalVoteError::InsufficientPermissions.into());
        }

        let user_info = if self.is_hidden() {
            None
        } else {
            Some(UserInfo {
                issuer: user_id,
                participant_id,
            })
        };
        let entry = ProtocolEntry::new(VoteEvent::Issue { user_info, issue });
        self.protocol.push(entry);

        Ok(())
    }

    /// Record that a participant has joined during an ongoing vote. Participants are only recorded
    /// if they are allowed to take part in the vote.
    pub fn participant_joined(&mut self, user_id: UserId, participant_id: ParticipantId) {
        if !self.parameters.allowed_users.contains(&user_id) {
            return;
        }

        let user_info = if self.is_hidden() {
            None
        } else {
            Some(UserInfo {
                issuer: user_id,
                participant_id,
            })
        };
        let entry = ProtocolEntry::new(VoteEvent::UserJoined { user_info });
        self.protocol.push(entry);
    }

    /// Record that a participant has disconnected during an ongoing vote. Participants are only
    /// recorded if they are allowed to take part in the vote.
    pub fn participant_disconnected(&mut self, user_id: UserId, participant_id: ParticipantId) {
        if !self.parameters.allowed_users.contains(&user_id) {
            return;
        }

        let user_info = if self.is_hidden() {
            None
        } else {
            Some(UserInfo {
                issuer: user_id,
                participant_id,
            })
        };
        let entry = ProtocolEntry::new(VoteEvent::UserLeft { user_info });
        self.protocol.push(entry);
    }

    /// Cancel the vote and record the reason in the protocol.
    pub fn cancel(mut self, issuer: UserId, reason: CancelReason) -> CanceledVote {
        self.stop_timeout(LegalVoteLoopback::VoteCancelled);

        let end_time = Utc::now();
        let entry = ProtocolEntry::new_with_time(end_time, VoteEvent::Cancel { issuer, reason });
        self.protocol.push(entry);

        CanceledVote {
            parameters: self.parameters,
            end_time,
            protocol: self.protocol,
        }
    }

    /// End the vote, verify the results and record them in the protocol.
    pub fn end(mut self, stop_kind: StopKind) -> CompletedVote {
        self.stop_timeout(LegalVoteLoopback::VoteEnded);

        let end_time = Utc::now();
        let entry = ProtocolEntry::new_with_time(end_time, VoteEvent::Stop(stop_kind));
        self.protocol.push(entry);

        let voting_record = match try_into_voting_record(&self.protocol) {
            Ok(voting_record) => voting_record,
            Err(err) => {
                tracing::warn!("Failed to generate `VotingRecord`: {err:?}");
                return CompletedVote {
                    parameters: self.parameters,
                    end_time,
                    protocol: self.protocol,
                    results: event::FinalResults::Invalid(Invalid::ProtocolInconsistent),
                };
            }
        };

        // Verify that the amount of votes and the tally are consistent
        let mut protocol_tally = Tally {
            yes: 0,
            no: 0,
            abstain: self.parameters.inner.enable_abstain.then_some(0),
        };
        let mut total_votes = 0;

        for vote_option in voting_record.vote_option_list() {
            total_votes += 1;

            match vote_option {
                VoteOption::Yes => protocol_tally.yes += 1,
                VoteOption::No => protocol_tally.no += 1,
                VoteOption::Abstain => {
                    if let Some(abstain) = &mut protocol_tally.abstain {
                        *abstain += 1;
                    } else {
                        return CompletedVote {
                            parameters: self.parameters,
                            end_time,
                            protocol: self.protocol,
                            results: event::FinalResults::Invalid(Invalid::AbstainDisabled),
                        };
                    }
                }
            }
        }

        let results = if protocol_tally == self.tally && total_votes <= self.parameters.max_votes {
            self.protocol
                .push(ProtocolEntry::new(VoteEvent::FinalResults(
                    FinalResults::Valid(protocol_tally),
                )));
            event::FinalResults::Valid(Results {
                tally: protocol_tally,
                voting_record,
            })
        } else {
            let invalid = Invalid::VoteCountInconsistent;
            self.protocol
                .push(ProtocolEntry::new(VoteEvent::FinalResults(
                    FinalResults::Invalid(invalid),
                )));
            event::FinalResults::Invalid(invalid)
        };

        CompletedVote {
            parameters: self.parameters,
            end_time,
            protocol: self.protocol,
            results,
        }
    }

    pub fn id(&self) -> LegalVoteId {
        self.id
    }

    pub fn parameters(&self) -> &Parameters {
        &self.parameters
    }

    pub fn tally(&self) -> Tally {
        self.tally
    }

    pub fn protocol(&self) -> &Vec<ProtocolEntry> {
        &self.protocol
    }

    pub fn is_live(&self) -> bool {
        self.parameters.inner.live
    }

    pub fn is_hidden(&self) -> bool {
        self.parameters.inner.pseudonymous
    }

    pub fn should_close(&self) -> bool {
        self.parameters.inner.auto_close && self.allowed_tokens.is_empty()
    }

    /// Stop the timeout loopback if it exists
    fn stop_timeout(&mut self, loopback: LegalVoteLoopback) {
        if let Some(cancel) = self.timeout_cancel.take()
            && cancel.send(loopback).is_err()
        {
            tracing::debug!("Vote timeout cancel sender has been dropped");
        }
    }
}
