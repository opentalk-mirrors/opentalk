// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{BTreeMap, HashMap};

use anyhow::Context;
use chat_id::{ChatId, PrivateChatId};
use opentalk_roomserver_signaling::{
    module_context::ModuleContext,
    signaling_module::{
        ModuleJoinData, ModuleSwitchData, NoOp, SignalingModule, SignalingModuleDescription,
        SignalingModuleFeatureDescription, SignalingModuleInitData,
    },
};
use opentalk_roomserver_types::{
    breakout::{BreakoutRoom, breakout_id::BreakoutId},
    connection_id::ConnectionId,
    room_kind::RoomKind,
    signaling::module_error::SignalingModuleError,
};
use opentalk_roomserver_types_chat::{
    CHAT_MODULE_ID, ChatSettings, MessageId, Scope,
    command::ChatCommand,
    event::{ChatError, ChatEvent},
    peer_state::ChatPeerState,
    state::{
        BreakoutHistory, CHAT_CHUNK_SIZE, ChatChunk, ChatState, PrivateHistory, StoredMessage,
    },
};
use opentalk_types_common::{modules::ModuleId, time::Timestamp};
use opentalk_types_signaling::ParticipantId;

use crate::rate_limit::{RateLimit, RateLimitState};

pub mod chat_id;
mod rate_limit;

const MIN_SEARCH_TERM_LENGTH: usize = 2;

#[derive(Debug)]
pub struct ChatModule {
    enabled: bool,

    history: HashMap<ChatId, Vec<StoredMessage>>,

    /// Records for each participant in which chat they are participating and up
    /// until which time they read messages.
    chat_state: HashMap<ParticipantId, HashMap<ChatId, Option<Timestamp>>>,
    rate_limit: Option<RateLimit>,
}

impl SignalingModuleDescription for ChatModule {
    const MODULE_ID: ModuleId = CHAT_MODULE_ID;
    const DESCRIPTION: &'static str = "Handles room chat functionality";
    const FEATURES: &[SignalingModuleFeatureDescription] = &[];

    type Settings = ChatSettings;
}

impl SignalingModule for ChatModule {
    const NAMESPACE: ModuleId = CHAT_MODULE_ID;

    type Incoming = ChatCommand;

    type Outgoing = ChatEvent;

    type Internal = NoOp;

    type Loopback = ();

    type JoinInfo = ChatState;

    type PeerJoinInfo = ChatPeerState;

    type Error = ChatError;

    fn init(init_data: SignalingModuleInitData) -> Option<Self> {
        let rate_limit = init_data
            .room_parameters
            .module_settings
            .get::<ChatSettings>()
            .ok()
            .flatten()
            .and_then(|settings| settings.rate_limit)
            .map(RateLimit::new);

        Some(Self {
            enabled: true,
            history: HashMap::default(),
            chat_state: HashMap::default(),
            rate_limit,
        })
    }

    fn on_participant_joined(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        p_joined: ParticipantId,
        connection_id: ConnectionId,
        _is_first_connection: bool,
    ) -> Result<ModuleJoinData<Self>, SignalingModuleError<Self::Error>> {
        let mut join_info = ModuleJoinData {
            join_success: Some(self.chat_state_latest_chunks_for_participant(p_joined, ctx.room)),
            ..Default::default()
        };

        join_info
            .peer_events
            .insert_for_all(ctx, ChatPeerState { groups: Vec::new() })?;

        if let Some(rate_limit) = &mut self.rate_limit {
            rate_limit.insert_connection(connection_id);
        }

        Ok(join_info)
    }

    fn on_participant_disconnected(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        _participant_id: ParticipantId,
        connection_id: ConnectionId,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        if let Some(rate_limit) = &mut self.rate_limit {
            rate_limit.remove_connection(connection_id);
        }
        Ok(())
    }

    fn on_websocket_message(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        command: Self::Incoming,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        match command {
            ChatCommand::EnableChat => {
                self.set_chat_state(ctx, participant_id, true)?;
            }
            ChatCommand::DisableChat => {
                self.set_chat_state(ctx, participant_id, false)?;
            }
            ChatCommand::SendMessage { content, scope } => {
                self.send_message(ctx, participant_id, connection_id, content, scope)?;
            }
            ChatCommand::GetHistoryChunk {
                message_index,
                scope,
            } => {
                self.get_history_chunk(ctx, participant_id, message_index, scope)?;
            }
            ChatCommand::ClearHistory => {
                self.clear_messages(ctx, participant_id)?;
            }
            ChatCommand::SetLastSeenTimestamp { scope, timestamp } => {
                self.set_last_seen_timestamp(ctx, participant_id, scope, timestamp)?;
            }
            ChatCommand::SearchHistory {
                scope,
                term,
                message_index,
            } => {
                self.search_history(ctx, participant_id, scope, &term, message_index)?;
            }
        }

        Ok(())
    }

    fn on_breakout_start(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
        rooms: &[BreakoutRoom],
        _duration: Option<std::time::Duration>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        for room in rooms {
            self.history.insert(ChatId::Breakout(room.id), Vec::new());
        }

        Ok(())
    }

    fn on_breakout_switch(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        participant_id: ParticipantId,
        _old_room: RoomKind,
        new_room: RoomKind,
    ) -> Result<ModuleSwitchData<Self>, SignalingModuleError<Self::Error>> {
        let mut switch_info = ModuleSwitchData::<Self>::new();
        let chat_state = self.chat_state_latest_chunks_for_participant(participant_id, new_room);

        let connections = ctx
            .participants
            .connected()
            .get(&participant_id)
            .context("failed to get participant state")?
            .connections();

        for conn_id in connections {
            switch_info
                .switch_success
                .insert(conn_id, Some(chat_state.clone()));
        }

        Ok(switch_info)
    }

    fn on_breakout_closed(
        &mut self,
        _ctx: &mut ModuleContext<'_, Self>,
    ) -> Result<(), SignalingModuleError<Self::Error>> {
        // remove all breakout chat histories
        self.history
            .retain(|id, _| !matches!(id, ChatId::Breakout(_)));

        Ok(())
    }
}

impl ChatModule {
    fn chat_state_latest_chunks_for_participant(
        &mut self,
        participant: ParticipantId,
        room_kind: RoomKind,
    ) -> ChatState {
        let (breakout_room_history, last_seen_timestamp_breakout) = match room_kind {
            RoomKind::Main => (None, None),
            RoomKind::Breakout(breakout_id) => {
                let history = self.history.get(&ChatId::Breakout(breakout_id));
                let chunk = Self::get_latest_chunk_or_default(history);

                let last_seen = self.last_seen_breakout(participant, breakout_id);

                (Some(chunk), last_seen)
            }
        };

        let global_history = self.history.get(&ChatId::Global);
        let global_history = Self::get_latest_chunk_or_default(global_history);

        ChatState {
            enabled: self.enabled,
            global_history,
            breakout_room_history,
            private_history: self.private_chat_histories_latest_chunk(participant),
            last_seen_timestamp_global: self.last_seen_global(participant),
            last_seen_timestamp_breakout,
            last_seen_timestamps_private: self.last_seen_timestamps_private(participant),
            last_seen_timestamps_group: Default::default(),
        }
    }

    fn last_seen_timestamps_private(
        &mut self,
        participant: ParticipantId,
    ) -> BTreeMap<ParticipantId, Timestamp> {
        let chats = self.chat_state.entry(participant).or_default();
        let mut last_seen_timestamps_private = BTreeMap::new();
        for (chat_id, timestamp) in chats {
            if let (ChatId::Private(private_id), Some(timestamp)) = (chat_id, timestamp) {
                last_seen_timestamps_private.insert(private_id.other(participant), *timestamp);
            }
        }
        last_seen_timestamps_private
    }

    fn private_chat_histories_latest_chunk(
        &mut self,
        participant: ParticipantId,
    ) -> Vec<PrivateHistory> {
        if let Some(chat_states) = self.chat_state.get(&participant) {
            chat_states
                .keys()
                .filter_map(|id| {
                    id.as_private().and_then(|private_id| {
                        self.private_history_latest_chunk(participant, private_id)
                    })
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    fn last_seen_global(&mut self, participant: ParticipantId) -> Option<Timestamp> {
        self.chat_state
            .entry(participant)
            .or_default()
            .get(&ChatId::Global)
            .copied()
            .flatten()
    }

    fn last_seen_breakout(
        &mut self,
        participant: ParticipantId,
        breakout_id: BreakoutId,
    ) -> Option<Timestamp> {
        self.chat_state
            .entry(participant)
            .or_default()
            .get(&ChatId::Breakout(breakout_id))
            .copied()
            .flatten()
    }

    fn private_history_latest_chunk(
        &self,
        participant: ParticipantId,
        chat_id: PrivateChatId,
    ) -> Option<PrivateHistory> {
        let Some(history) = self.history.get(&ChatId::Private(chat_id)) else {
            tracing::debug!("No private history found for chat: {chat_id:?}");
            return None;
        };
        tracing::debug!(
            "Private history found for chat `{chat_id:?}` with {} messages",
            history.len()
        );

        let chunk = Self::get_latest_chunk(history);
        let correspondent = chat_id.other(participant);

        Some(PrivateHistory {
            correspondent,
            history: chunk,
        })
    }

    /// Retrieves the latest [`ChatChunk`] from the provided `history` or returns a
    /// default (empty) [`ChatChunk`] when `history` is `None`.
    fn get_latest_chunk_or_default(history: Option<&Vec<StoredMessage>>) -> ChatChunk {
        if let Some(history) = history {
            Self::get_latest_chunk(history)
        } else {
            ChatChunk::default()
        }
    }

    /// Retrieves the latest [`ChatChunk`] from the provided `history`
    fn get_latest_chunk(history: &[StoredMessage]) -> ChatChunk {
        let message_index = history.len().saturating_sub(1);
        Self::get_chunk(history, message_index)
    }

    /// Retrieves the chunk that starts at the message with index `message_index`
    /// or a default [`ChatChunk`] when `history` is [`None`].
    fn get_chunk_or_default(history: Option<&Vec<StoredMessage>>, message_index: u32) -> ChatChunk {
        if let Some(history) = history {
            Self::get_chunk(history, message_index as usize)
        } else {
            ChatChunk::default()
        }
    }

    /// Retrieves the chunk that starts at the message with index `message_index`
    /// from the messages that match the search term `term` in the provided
    /// `history` or a default [`ChatChunk`] when `history` is [`None`].
    fn search_history_chunked(
        term: &str,
        history: Option<&Vec<StoredMessage>>,
        message_index: Option<u32>,
    ) -> ChatChunk {
        let Some(history) = history else {
            return ChatChunk::default();
        };

        let filtered: Vec<&StoredMessage> = history
            .iter()
            .filter(|msg| msg.content.contains(term))
            .collect();
        // Not using `get_chunk()` here because that would require us to copy all
        // messages matching the search term instead of only those in the chunk.
        let message_index = message_index
            .map(|m| m as usize)
            .unwrap_or(filtered.len().saturating_sub(1));
        let start = message_index.saturating_sub(CHAT_CHUNK_SIZE as usize - 1);

        let Some(messages) = filtered.get(start..=message_index) else {
            return ChatChunk::default();
        };

        if messages.is_empty() {
            return ChatChunk::default();
        }

        ChatChunk {
            messages: messages.iter().map(|msg| (*msg).clone()).collect(),
            next_index: start.try_into().ok().and_then(|s: u32| s.checked_sub(1)),
        }
    }

    /// Retrieves the chunk that starts at the message with index `message_index`.
    fn get_chunk(history: &[StoredMessage], message_index: usize) -> ChatChunk {
        let start = message_index.saturating_sub(CHAT_CHUNK_SIZE as usize - 1);
        let Some(messages) = history.get(start..=message_index) else {
            return ChatChunk::default();
        };

        ChatChunk {
            messages: messages.to_vec(),
            next_index: start.try_into().ok().and_then(|s: u32| s.checked_sub(1)),
        }
    }

    fn set_chat_state(
        &mut self,
        ctx: &mut ModuleContext<'_, ChatModule>,
        participant: ParticipantId,
        enabled: bool,
    ) -> Result<(), SignalingModuleError<<ChatModule as SignalingModule>::Error>> {
        if !ctx.is_moderator(participant) {
            return Err(ChatError::InsufficientPermissions.into());
        }

        self.enabled = enabled;
        let msg = if enabled {
            ChatEvent::ChatEnabled {
                issued_by: participant,
            }
        } else {
            ChatEvent::ChatDisabled {
                issued_by: participant,
            }
        };

        ctx.send_ws_message(ctx.participants.connected().ids(), msg)?;
        Ok(())
    }

    fn send_message(
        &mut self,
        ctx: &mut ModuleContext<'_, ChatModule>,
        participant_id: ParticipantId,
        connection_id: ConnectionId,
        content: String,
        scope: Scope,
    ) -> Result<(), SignalingModuleError<<ChatModule as SignalingModule>::Error>> {
        if let Some(rate_limit) = &mut self.rate_limit {
            match rate_limit.consume_token(connection_id) {
                RateLimitState::TooManyRequests { retry_after_ms } => {
                    // The moderators are not informed about rate limited messages if the sender is
                    // the room owner, as the room owner can not be kicked.
                    if !ctx.is_room_owner(participant_id) {
                        ctx.send_ws_message(
                            ctx.participants
                                .in_room(ctx.room)
                                .moderators()
                                .ids()
                                .filter(|id| *id != participant_id),
                            ChatEvent::ParticipantRateLimited { participant_id },
                        )?;
                    }

                    return Err(ChatError::TooManyRequests { retry_after_ms }.into());
                }
                RateLimitState::SlowDown {
                    recommended_wait_ms,
                } => {
                    ctx.send_ws_message_to_connections(
                        [connection_id],
                        ChatEvent::SlowDown {
                            recommended_wait_ms,
                        },
                    )?;
                }
                RateLimitState::Ok => {}
            }
        }

        if !self.enabled {
            return Err(ChatError::ChatDisabled.into());
        }

        if let Scope::Breakout(breakout_id) = scope {
            // deny messages to other breakout rooms
            let RoomKind::Breakout(current_breakout_id) = ctx.room else {
                return Err(ChatError::InvalidBreakoutScope.into());
            };

            if current_breakout_id != breakout_id {
                return Err(ChatError::InvalidBreakoutScope.into());
            }
        }

        // Ensure target participant exists
        if let Scope::Private(participant_id) = scope
            && !ctx
                .participants
                .all_unfiltered
                .contains_key(&participant_id)
        {
            return Err(ChatError::UnknownParticipant.into());
        }

        let id = MessageId::generate();
        let stored_msg = StoredMessage {
            id,
            source: participant_id,
            content: content.clone(),
            scope: scope.clone(),
            timestamp: Timestamp::now(),
        };
        let chat_id = ChatId::from_scope_and_source(scope.clone(), participant_id);

        self.history
            .entry(chat_id.clone())
            .or_default()
            .push(stored_msg);

        // ensure participation in the chat is recorded
        self.chat_state
            .entry(participant_id)
            .or_default()
            .entry(chat_id.clone())
            .or_insert(None);

        // if this is a private chat, we also record the chat for the other participant
        if let ChatId::Private(id) = &chat_id {
            self.chat_state
                .entry(id.other(participant_id))
                .or_default()
                .entry(chat_id.clone())
                .or_insert(None);
        }

        match &chat_id {
            ChatId::Global => {
                ctx.send_ws_message(
                    ctx.participants.connected().ids(),
                    ChatEvent::MessageSent {
                        id,
                        source: participant_id,
                        content,
                        scope,
                    },
                )?;
            }
            ChatId::Breakout(breakout_id) => {
                ctx.send_ws_message(
                    ctx.participants
                        .connected()
                        .room(RoomKind::Breakout(*breakout_id))
                        .ids(),
                    ChatEvent::MessageSent {
                        id,
                        source: participant_id,
                        content,
                        scope,
                    },
                )?;
            }
            ChatId::Private(private_chat_id) => {
                // Since the Scope is relative to recipient, we need to calculate
                // individual scopes for each recipient of the message.
                for recipient in private_chat_id.participants() {
                    ctx.send_ws_message(
                        [recipient],
                        ChatEvent::MessageSent {
                            id,
                            source: participant_id,
                            content: content.clone(),
                            scope: private_chat_id.to_scope(recipient),
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Sends the [`ChatChunk`] starting at `message_index` in the history of the
    /// requested `scope` to the participant
    fn get_history_chunk(
        &self,
        ctx: &ModuleContext<'_, ChatModule>,
        sender: ParticipantId,
        message_index: u32,
        scope: Scope,
    ) -> Result<(), SignalingModuleError<ChatError>> {
        if !Self::can_access_scope(ctx, sender, &scope)? {
            return Err(ChatError::InsufficientPermissions.into());
        }

        let chat_id = ChatId::from_scope_and_source(scope.clone(), sender);
        let history = self.history.get(&chat_id);
        let history = Self::get_chunk_or_default(history, message_index);

        let event = match scope {
            Scope::Global => ChatEvent::RoomChatHistoryChunk { history },
            Scope::Breakout(breakout_id) => ChatEvent::BreakoutChatHistoryChunk(BreakoutHistory {
                breakout_id,
                history,
            }),
            Scope::Private(participant_id) => ChatEvent::PrivateChatHistoryChunk(PrivateHistory {
                correspondent: participant_id,
                history,
            }),
        };

        ctx.send_ws_message([sender], event)?;

        Ok(())
    }

    fn can_access_scope(
        ctx: &ModuleContext<'_, ChatModule>,
        participant: ParticipantId,
        scope: &Scope,
    ) -> anyhow::Result<bool> {
        // Only moderators are allowed to access messages of other breakout rooms
        if let Scope::Breakout(breakout_id) = scope
            && !ctx.is_moderator(participant)
        {
            let room = ctx
                .participant_state(participant)
                .with_context(|| format!("Participant {participant} has no state"))?
                .room;
            return Ok(room == RoomKind::Breakout(*breakout_id));
        }
        Ok(true)
    }

    fn search_history(
        &mut self,
        ctx: &mut ModuleContext<'_, Self>,
        sender: ParticipantId,
        scope: Scope,
        term: &str,
        message_index: Option<u32>,
    ) -> Result<(), SignalingModuleError<ChatError>> {
        if term.len() < MIN_SEARCH_TERM_LENGTH {
            return Err(ChatError::InvalidSearchTermLength {
                min: MIN_SEARCH_TERM_LENGTH,
            }
            .into());
        }

        if !Self::can_access_scope(ctx, sender, &scope)? {
            return Err(ChatError::InsufficientPermissions.into());
        }

        let chat_id = ChatId::from_scope_and_source(scope.clone(), sender);
        let history = self.history.get(&chat_id);
        let history = Self::search_history_chunked(term, history, message_index);

        ctx.send_ws_message(
            [sender],
            ChatEvent::SearchResults {
                matches: history,
                scope,
            },
        )?;

        Ok(())
    }

    fn clear_messages(
        &mut self,
        ctx: &mut ModuleContext<'_, ChatModule>,
        participant: ParticipantId,
    ) -> Result<(), SignalingModuleError<<ChatModule as SignalingModule>::Error>> {
        if !ctx.is_moderator(participant) {
            return Err(ChatError::InsufficientPermissions.into());
        }

        self.history.remove(&ChatId::Global);

        ctx.send_ws_message(
            ctx.participants.connected().ids(),
            ChatEvent::HistoryCleared {
                issued_by: participant,
            },
        )?;
        Ok(())
    }

    fn set_last_seen_timestamp(
        &mut self,
        ctx: &mut ModuleContext<'_, ChatModule>,
        participant_id: ParticipantId,
        scope: Scope,
        timestamp: Timestamp,
    ) -> Result<(), SignalingModuleError<<ChatModule as SignalingModule>::Error>> {
        let chat_id = ChatId::from_scope_and_source(scope.clone(), participant_id);
        self.chat_state
            .entry(participant_id)
            .or_default()
            .insert(chat_id, Some(timestamp));

        ctx.send_ws_message(
            [participant_id],
            ChatEvent::SetLastSeenTimestamp { scope, timestamp },
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use opentalk_roomserver_signaling::signaling_module::SignalingModuleDescription as _;

    use crate::ChatModule;

    #[test]
    fn serialize_example_config() {
        let produced = toml::to_string_pretty(&ChatModule::example_config()).unwrap();

        assert_snapshot!(produced, @"
        [chat.rate_limit]
        slow_down_threshold = 0.75
        token_bucket_size = 30
        tokens_per_second = 10
        ");
    }
}
