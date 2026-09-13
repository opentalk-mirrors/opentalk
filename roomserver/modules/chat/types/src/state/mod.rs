// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling state for the `chat` namespace

mod breakout_history;
mod chat_chunk;
mod chat_state;
mod private_history;
mod stored_message;

pub use breakout_history::BreakoutHistory;
pub use chat_chunk::{CHAT_CHUNK_SIZE, ChatChunk};
pub use chat_state::ChatState;
pub use private_history::PrivateHistory;
pub use stored_message::StoredMessage;
