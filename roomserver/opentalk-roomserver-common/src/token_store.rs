// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeSet, HashMap, hash_map::Entry},
    time::{Duration, Instant},
};

use opentalk_types_common::roomserver::Token;

const DEFAULT_EXPIRY: Duration = Duration::from_secs(30);

/// Information about a token and when it was created
///
/// Used to determine if a token has expired
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TokenExpiry {
    created_at: Instant,
    token: Token,
}

impl TokenExpiry {
    fn new_now(token: Token) -> Self {
        Self {
            created_at: Instant::now(),
            token,
        }
    }
}

/// Manages the active signaling tokens of the RoomServer
///
/// Expired tokens get cleaned up when the [`TokenStore`] is accessed
pub struct TokenStore<V> {
    /// The active tokens
    tokens: HashMap<Token, V>,
    /// A set to track the expiry of each token
    expiry_set: BTreeSet<TokenExpiry>,
    /// The duration for that a token is active
    expiry: Duration,
}

impl<V> Default for TokenStore<V> {
    fn default() -> Self {
        Self::new_with_expiry(DEFAULT_EXPIRY)
    }
}

impl<V> TokenStore<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_expiry(expiry: Duration) -> Self {
        Self {
            tokens: HashMap::new(),
            expiry_set: BTreeSet::new(),
            expiry,
        }
    }

    pub fn create_token(&mut self, value: V) -> Token {
        self.remove_expired_entries();

        // loop to avoid the uuid collision error branch
        let token = loop {
            let token = Token::generate();

            match self.tokens.entry(token) {
                Entry::Occupied(_) => {}
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(value);
                    break token;
                }
            }
        };

        self.expiry_set.insert(TokenExpiry::new_now(token));

        token
    }

    pub fn consume_token(&mut self, token: &Token) -> Option<V> {
        self.remove_expired_entries();
        self.tokens.remove(token)
    }

    pub fn peek_token(&mut self, token: &Token) -> Option<&V> {
        self.remove_expired_entries();
        self.tokens.get(token)
    }

    fn remove_expired_entries(&mut self) {
        let now = Instant::now();

        while let Some(&entry) = self.expiry_set.first() {
            let duration_since_creation = now.duration_since(entry.created_at);

            if duration_since_creation >= self.expiry {
                self.expiry_set.remove(&entry);
                self.tokens.remove(&entry.token);
            } else {
                // The oldest key is not yet expired
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        str::FromStr,
        thread,
        time::{Duration, Instant},
    };

    use opentalk_roomserver_types::{
        client_parameters::{ClientKind, ClientParameters, Role},
        public_user_profile::PublicUserProfile,
        signaling::signaling_context::SignalingClientContext,
    };
    use opentalk_types_common::{
        rooms::RoomId,
        roomserver::{DeviceSecret, Token},
        time::TimeZone,
        users::{UserId, UserInfo},
        utils::ExampleData,
    };
    use pretty_assertions::assert_eq;

    use super::{TokenExpiry, TokenStore};

    fn build_test_context(i: u128) -> SignalingClientContext {
        SignalingClientContext {
            room_id: RoomId::from_u128(i),
            client_parameters: ClientParameters {
                device_secret: DeviceSecret::from_str(&format!("LongEnoughToBeADeviceSecret{i}"))
                    .expect("Valid device secret"),
                kind: ClientKind::Registered {
                    profile: PublicUserProfile {
                        id: UserId::from_u128(i),
                        email: "alice@example.com".to_string(),
                        user_info: UserInfo {
                            title: "".parse().expect("valid user title"),
                            firstname: "Alice".to_string(),
                            lastname: "Adams".to_string(),
                            display_name: "Alice Adams".parse().expect("valid display name"),
                            avatar_url:
                                "https://gravatar.com/avatar/c160f8cc69a4f0bf2b0362752353d060"
                                    .to_string(),
                        },
                        timezone: TimeZone::example_data(),
                    },
                },
                role: Role::User,
            },
        }
    }

    #[test]
    fn test_expiry() {
        let mut store = TokenStore::new_with_expiry(Duration::from_millis(500));

        let context1 = build_test_context(1);
        let token1 = store.create_token(context1.clone());

        let context2 = build_test_context(2);
        let token2 = store.create_token(context2.clone());

        // get the right context for token1
        assert_eq!(store.consume_token(&token1), Some(context1));
        // ensure the token was consumed
        assert_eq!(None, store.consume_token(&token1));

        thread::sleep(Duration::from_millis(750));

        //ensure the second token expired
        assert_eq!(None, store.consume_token(&token2));
    }

    /// The sorting of [Token] depends on the order of the structs fields. This
    /// test shall ensure that the tokens are ordered by created-at time first.
    #[test]
    fn test_token_sorting() {
        let tokens = [
            // expired tokens
            TokenExpiry {
                token: Token::from_u128(5),
                created_at: Instant::now().checked_sub(Duration::from_secs(63)).unwrap(),
            },
            TokenExpiry {
                token: Token::from_u128(4),
                created_at: Instant::now().checked_sub(Duration::from_secs(62)).unwrap(),
            },
            TokenExpiry {
                token: Token::from_u128(3),
                created_at: Instant::now().checked_sub(Duration::from_secs(61)).unwrap(),
            },
            // valid tokens
            TokenExpiry {
                token: Token::from_u128(2),
                created_at: Instant::now().checked_add(Duration::from_secs(61)).unwrap(),
            },
            TokenExpiry {
                token: Token::from_u128(1),
                created_at: Instant::now().checked_add(Duration::from_secs(62)).unwrap(),
            },
            TokenExpiry {
                token: Token::from_u128(0),
                created_at: Instant::now().checked_add(Duration::from_secs(63)).unwrap(),
            },
        ]
        .to_vec();
        let mut sorted_tokens = tokens.clone();
        sorted_tokens.sort();

        assert_eq!(sorted_tokens, tokens);
    }
}
