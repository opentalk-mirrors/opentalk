// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling vote for the `legal-vote` namespace.

mod legal_vote_id;
mod stop_kind;
mod vote_option;
mod vote_state;
mod vote_summary;

pub use legal_vote_id::LegalVoteId;
pub use stop_kind::StopKind;
pub use vote_option::VoteOption;
pub use vote_state::VoteState;
pub use vote_summary::VoteSummary;
