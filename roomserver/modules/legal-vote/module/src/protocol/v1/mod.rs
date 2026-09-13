// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

//! Signaling protocol v1 for the `legal-vote` namespace.
mod final_results;
mod protocol_entry;
mod stop_kind;
mod user_info;
mod vote_event;

pub use final_results::FinalResults;
pub use protocol_entry::ProtocolEntry;
pub use stop_kind::StopKind;
pub use user_info::UserInfo;
pub use vote_event::VoteEvent;
