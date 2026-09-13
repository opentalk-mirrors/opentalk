// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::collections::{VecDeque, vec_deque::Iter};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageHistory {
    /// Maximum number of entries that should be stored.
    ///
    /// The limit is enforced when pushing new messages. If the limit was reduced,
    /// the history might be bigger than the limit.
    pub limit: usize,

    /// Message history.
    ///
    /// A [`VecDeque`] was chosen since we remove messages incase the limit is reached and append
    /// messages on the other end.
    messages: VecDeque<HistoryEntry>,
}

impl MessageHistory {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            messages: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.messages.get(index)
    }

    pub fn move_front(&mut self, index: usize) {
        let item = self.messages.remove(index);
        if let Some(item) = item {
            self.messages.push_front(item);
        }
    }

    pub fn push(&mut self, message: String) {
        if self.messages.len() >= self.limit {
            self.messages.truncate(self.limit);
        }
        // don't push duplicates
        if self.messages.front().is_none_or(|f| f.text != message) {
            self.messages.push_front(HistoryEntry::new(message));
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.messages.remove(index);
    }

    pub fn iter(&self) -> Iter<'_, HistoryEntry> {
        self.messages.iter()
    }
}

impl Default for MessageHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    text: String,

    /// cached serialization (if the [`Self::text`] is valid json)
    json_cache: Option<serde_json::Value>,
}

// manual serialize since we serialize the entry as a plain string and ignore
// the other fields.
impl Serialize for HistoryEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::serialize(&self.text, serializer)
    }
}

// manual serialize from a string. Call [`HistoryEntry::new`] which will try to
// serialize the string as json.
impl<'de> Deserialize<'de> for HistoryEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        Ok(Self::new(text))
    }
}

impl HistoryEntry {
    pub fn new(text: String) -> Self {
        let json_cache: Option<Value> = serde_json::from_str(&text).ok();
        Self { text, json_cache }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the parsed json if the text was valid json.
    /// The [Value] is stored internally, so that the text is not parsed repeatedly.
    pub fn json(&self) -> Option<&Value> {
        self.json_cache.as_ref()
    }
}
