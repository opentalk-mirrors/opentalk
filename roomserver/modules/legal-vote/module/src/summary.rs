// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use anyhow::Context as _;
use opentalk_roomserver_types_legal_vote::{
    event::Results,
    vote::{VoteState, VoteSummary},
};

use crate::{
    protocol::v1::{FinalResults, ProtocolEntry, VoteEvent},
    try_into_voting_record,
    vote::ActiveVote,
};

pub fn from_history<'a>(
    active_vote: Option<&'a ActiveVote>,
    history: impl Iterator<Item = &'a Vec<ProtocolEntry>>,
) -> anyhow::Result<Vec<VoteSummary>> {
    history
        .chain(active_vote.iter().map(|vote| vote.protocol()))
        .map(from_protocol)
        .collect()
}

pub fn from_protocol(protocol: &Vec<ProtocolEntry>) -> anyhow::Result<VoteSummary> {
    let mut parameters = None;
    let mut state = None;
    let mut end_time = None;
    let mut stop_kind = None;

    for entry in protocol {
        match entry.event.clone() {
            VoteEvent::Start {
                parameters: params, ..
            } => {
                parameters = Some(params);
                state = Some(VoteState::Started);
            }

            VoteEvent::Stop(kind) => {
                stop_kind = Some(kind);
                end_time = entry.timestamp;
            }

            VoteEvent::Cancel { issuer, reason } => {
                state = Some(VoteState::Canceled { issuer, reason });
                end_time = entry.timestamp;
            }

            VoteEvent::FinalResults(results) => match results {
                FinalResults::Valid(tally) => {
                    let voting_record = try_into_voting_record(protocol)?;
                    let stop_kind = stop_kind
                        .context(
                            "Missing `Stop` entry before `FinalResults` in legal vote protocol",
                        )?
                        .into();

                    state = Some(VoteState::Finished {
                        stop_kind,
                        results: Results {
                            tally,
                            voting_record,
                        },
                    });
                }

                FinalResults::Invalid(reason) => {
                    state = Some(VoteState::Invalid(reason));
                }
            },
            _ => {}
        }
    }

    let parameters = *parameters.context("Missing `Start` in legal vote protocol")?;
    let state = state.context("Missing `VoteState` in legal vote protocol")?;

    Ok(VoteSummary {
        parameters,
        state,
        end_time,
    })
}
