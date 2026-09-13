// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::collections::{HashMap, HashSet};

use opentalk_roomserver_signaling::module_context::ModuleContext;
use opentalk_roomserver_types_legal_vote::{event::LegalVoteError, token::Token};
use opentalk_types_common::users::UserId;
use opentalk_types_signaling::ParticipantId;

use crate::LegalVoteModule;

pub struct UserTokens {
    pub participant_tokens: HashMap<ParticipantId, Token>,
    pub allowed_users: HashSet<UserId>,
}

impl UserTokens {
    pub fn try_generate(
        ctx: &mut ModuleContext<'_, LegalVoteModule>,
        allowed_participants: &HashSet<ParticipantId>,
    ) -> Result<Self, LegalVoteError> {
        let mut invalid_participants = Vec::new();
        let mut participant_tokens = HashMap::new();
        let mut allowed_users = HashSet::new();

        for participant_id in allowed_participants {
            if let Some(user_id) = ctx
                .participants
                .in_room(ctx.room)
                .get(participant_id)
                .and_then(|state| state.kind.user_id())
            {
                participant_tokens
                    .entry(*participant_id)
                    .or_insert_with(Token::generate);
                allowed_users.insert(user_id);
            } else {
                invalid_participants.push(*participant_id);
            }
        }

        if !invalid_participants.is_empty() {
            return Err(LegalVoteError::IneligibleParticipants {
                participants: invalid_participants,
            });
        }

        Ok(Self {
            participant_tokens,
            allowed_users,
        })
    }
}
