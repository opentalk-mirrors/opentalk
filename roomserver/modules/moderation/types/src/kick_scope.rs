// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_roomserver_types::{client_parameters::ClientKind, room_parameters::WaitingRoom};
use opentalk_types_signaling::Role;
use serde::{Deserialize, Serialize};

/// The scope of users to be kicked from the room
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kick_scope", rename_all = "snake_case")]
pub enum KickScope {
    /// Only kick guests from the room
    Guests,

    /// Kick both users and guests from the room but not moderators
    UsersAndGuests,

    /// Kick every participant from the room
    All,
}

impl KickScope {
    /// Query whether a specific role is kicked by the scope
    pub fn kicks(&self, role: Role, kind: &ClientKind) -> bool {
        match self {
            KickScope::Guests => {
                matches!(kind, ClientKind::Unregistered | ClientKind::CallIn)
            }
            KickScope::UsersAndGuests => !matches!(role, Role::Moderator),
            KickScope::All => true,
        }
    }
}

impl From<KickScope> for WaitingRoom {
    fn from(value: KickScope) -> Self {
        match value {
            KickScope::Guests => Self::ForGuests,
            KickScope::UsersAndGuests | KickScope::All => Self::ForEveryone,
        }
    }
}

#[cfg(test)]
mod tests {
    use opentalk_roomserver_types::{
        client_parameters::ClientKind, public_user_profile::PublicUserProfile,
    };
    use opentalk_types_common::utils::ExampleData;
    use opentalk_types_signaling::Role;

    use super::KickScope;

    #[test]
    fn guests_scope_kicks_call_in() {
        let kind = ClientKind::CallIn;

        assert!(KickScope::Guests.kicks(Role::User, &kind));
    }

    #[test]
    fn guests_scope_kicks_guest() {
        let kind = ClientKind::Unregistered;

        assert!(KickScope::Guests.kicks(Role::User, &kind));
    }

    #[test]
    fn guests_scope_does_not_kick_registered() {
        let kind = ClientKind::Registered {
            profile: PublicUserProfile::example_data(),
        };

        assert!(!KickScope::Guests.kicks(Role::User, &kind));
        assert!(!KickScope::Guests.kicks(Role::Moderator, &kind));
    }
}
