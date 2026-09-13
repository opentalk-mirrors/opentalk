// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The outcome of an upsert database request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpsertOutcome<T> {
    /// A new entry was inserted into the inventory.
    Inserted(T),

    /// An existing entry was updated in the inventory.
    Updated(T),
}

impl<T> UpsertOutcome<T> {
    /// Get the inner type.
    pub fn into_inner(self) -> T {
        match self {
            UpsertOutcome::Inserted(t) => t,
            UpsertOutcome::Updated(t) => t,
        }
    }

    /// Returns [`true`] if the variant is [`UpsertOutcome::Inserted`].
    pub fn is_inserted(&self) -> bool {
        matches!(self, Self::Inserted(_))
    }

    /// Returns [`true`] if the variant is [`UpsertOutcome::Inserted`].
    pub fn is_updated(&self) -> bool {
        matches!(self, Self::Updated(_))
    }
}
