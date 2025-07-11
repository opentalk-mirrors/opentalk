use std::collections::HashMap;

use anyhow::{Result, bail};
use types::{
    common::users::GroupName,
    signaling::{
        ParticipantId,
        chat::{
            MessageId, Scope,
            command::{ChatCommand, SendMessage},
            event::ChatEvent,
            state::ChatState,
        },
    },
};

use super::{OpenTalkEvent, Room};

#[derive(Debug)]
pub enum OpenTalkChatEvent {
    Enabled {
        /// Participant who enabled the chat
        issued_by: ParticipantId,
    },
    Disabled {
        /// Participant who disabled the chat
        issued_by: ParticipantId,
    },
    GlobalMessage(Message),
    GroupMessage {
        group_name: GroupName,
        message: Message,
    },
    PrivateMessage {
        participant_id: ParticipantId,
        message: Message,
    },
    Cleared {
        /// ID of the participant that cleared chat history
        issued_by: ParticipantId,
    },
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub source: ParticipantId,
    pub content: String,
}

pub struct ChatApi<'a> {
    pub(super) room: &'a mut Room,
}

impl<'a> ChatApi<'a> {
    fn state(&self) -> &RoomChatState {
        self.room.chat_state.as_ref().unwrap()
    }

    pub async fn send_message(&mut self, content: String, scope: Scope) -> Result<()> {
        if !self.state().enabled {
            bail!("chat is not enabled");
        }

        self.room
            .signaling
            .send(ChatCommand::SendMessage(SendMessage { content, scope }))
            .await
    }

    pub fn global_messages(&self) -> &[Message] {
        self.state()
            .messages
            .get(&Scope::Global)
            .map_or(&[], Vec::as_slice)
    }

    pub fn group_messages(&self, group_name: &GroupName) -> &[Message] {
        self.state()
            .messages
            .get(&Scope::Group(group_name.to_owned()))
            .map_or(&[], Vec::as_slice)
    }

    pub fn private_messages(&self, participant_id: &ParticipantId) -> &[Message] {
        self.state()
            .messages
            .get(&Scope::Private(participant_id.to_owned()))
            .map_or(&[], Vec::as_slice)
    }
}

#[derive(Debug)]
pub(super) struct RoomChatState {
    enabled: bool,
    messages: HashMap<Scope, Vec<Message>>,
}

impl RoomChatState {
    pub(super) fn new(state: ChatState) -> Self {
        Self {
            enabled: state.enabled,
            messages: HashMap::new(),
        }
    }

    pub(super) fn handle_websocket_event(&mut self, event: ChatEvent) -> Option<OpenTalkEvent> {
        match event {
            ChatEvent::ChatEnabled(chat_enabled) => {
                self.enabled = true;

                Some(OpenTalkEvent::Chat(OpenTalkChatEvent::Enabled {
                    issued_by: chat_enabled.issued_by,
                }))
            }
            ChatEvent::ChatDisabled(chat_disabled) => {
                self.enabled = false;

                Some(OpenTalkEvent::Chat(OpenTalkChatEvent::Disabled {
                    issued_by: chat_disabled.issued_by,
                }))
            }
            ChatEvent::MessageSent(message_sent) => {
                let message = Message {
                    id: message_sent.id,
                    source: message_sent.source,
                    content: message_sent.content,
                };

                self.messages
                    .entry(message_sent.scope.clone())
                    .or_default()
                    .push(message.clone());

                match message_sent.scope {
                    Scope::Global => Some(OpenTalkEvent::Chat(OpenTalkChatEvent::GlobalMessage(
                        message,
                    ))),
                    Scope::Group(group_name) => {
                        Some(OpenTalkEvent::Chat(OpenTalkChatEvent::GroupMessage {
                            group_name,
                            message,
                        }))
                    }
                    Scope::Private(participant_id) => {
                        Some(OpenTalkEvent::Chat(OpenTalkChatEvent::PrivateMessage {
                            participant_id,
                            message,
                        }))
                    }
                }
            }
            ChatEvent::HistoryCleared(history_cleared) => {
                self.messages.clear();

                Some(OpenTalkEvent::Chat(OpenTalkChatEvent::Cleared {
                    issued_by: history_cleared.issued_by,
                }))
            }
            ChatEvent::Error(error) => {
                log::error!("chat module received error: {error:?}");

                None
            }
        }
    }
}
