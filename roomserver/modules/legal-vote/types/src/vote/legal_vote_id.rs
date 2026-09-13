// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
// SPDX-License-Identifier: EUPL-1.2

use std::fmt;

use opentalk_types_common::module_resources::ModuleResourceId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique identifier for legal votes, built on `ModuleResourceId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LegalVoteId(ModuleResourceId);

impl LegalVoteId {
    /// Creates a zero `LegalVoteId` for testing.
    pub const fn nil() -> Self {
        Self::from(ModuleResourceId::nil())
    }

    /// Creates a `LegalVoteId` from a `u128` number.
    pub const fn from_u128(id: u128) -> Self {
        Self::from(ModuleResourceId::from_u128(id))
    }

    /// Generates a random `LegalVoteId` (requires `rand` feature).
    pub fn generate() -> Self {
        Self::from(ModuleResourceId::generate())
    }

    /// Creates a `LegalVoteId` from a `ModuleResourceId`.
    pub const fn from(inner: ModuleResourceId) -> Self {
        Self(inner)
    }

    /// Creates a `LegalVoteId` from a `Uuid`.
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self::from(ModuleResourceId::from_uuid(uuid))
    }

    /// Returns a reference to the inner `ModuleResourceId`.
    pub fn inner(&self) -> &ModuleResourceId {
        &self.0
    }

    /// Consumes the `LegalVoteId` and returns the inner `ModuleResourceId`.
    pub fn into_inner(self) -> ModuleResourceId {
        self.0
    }
}

impl From<ModuleResourceId> for LegalVoteId {
    fn from(value: ModuleResourceId) -> Self {
        Self::from(value)
    }
}

impl fmt::Display for LegalVoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;

    #[test]
    fn serialization() {
        let produced = serde_json::to_value(LegalVoteId::from_u128(1)).unwrap();

        let expected = json!("00000000-0000-0000-0000-000000000001");

        assert_eq!(produced, expected);
    }

    #[test]
    fn deserialization() {
        let produced: LegalVoteId =
            serde_json::from_value(json!("00000000-0000-0000-0000-000000000001")).unwrap();

        let expected = LegalVoteId::from_u128(1);

        assert_eq!(produced, expected);
    }
}
