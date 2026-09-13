// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling event messages for the `legal-vote` namespace.

mod error;
mod final_results;
mod legal_vote_event;
mod results;
mod stop_kind;
mod voting_record;

pub use error::LegalVoteError;
pub use final_results::FinalResults;
pub use legal_vote_event::LegalVoteEvent;
pub use results::Results;
pub use stop_kind::StopKind;
pub use voting_record::VotingRecord;
