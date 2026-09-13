// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types_legal_vote::parameters::Parameters;
use opentalk_types_common::users::UserId;

/// Represents the start of a vote, including the initiator and parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Start {
    /// The user ID of the initiator.
    pub issuer: UserId,

    /// The parameters for the vote.
    pub parameters: Parameters,
}
