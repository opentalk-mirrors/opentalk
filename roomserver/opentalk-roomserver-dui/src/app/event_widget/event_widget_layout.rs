// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui_json_tree::DefaultExpand;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct EventWidgetLayout {
    pub(crate) expanded: Expand,
}

impl EventWidgetLayout {
    pub fn new() -> Self {
        Self {
            expanded: Expand::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Expand {
    All,
    None,
    ToLevel(u8),
}

impl Expand {
    pub fn new() -> Self {
        Expand::ToLevel(1)
    }

    /// Returns `true` if the expand is [`ToLevel`].
    ///
    /// [`ToLevel`]: Expand::ToLevel
    #[must_use]
    pub fn is_to_level(&self) -> bool {
        matches!(self, Self::ToLevel(..))
    }
}

impl From<Expand> for DefaultExpand<'_> {
    fn from(value: Expand) -> Self {
        match value {
            Expand::All => DefaultExpand::All,
            Expand::None => DefaultExpand::None,
            Expand::ToLevel(l) => DefaultExpand::ToLevel(l),
        }
    }
}

impl Default for Expand {
    fn default() -> Self {
        Self::new()
    }
}
