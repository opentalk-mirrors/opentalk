// SPDX-License-Identifier: EUPL-1.2
//
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use serde::{Deserialize, Serialize};

use crate::breakout::breakout_id::BreakoutId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "id")]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub enum RoomKind {
    Main,
    Breakout(BreakoutId),
}
