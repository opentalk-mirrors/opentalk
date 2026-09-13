// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use derive_more::{AsRef, Display, From, FromStr, Into};
use serde::{Deserialize, Serialize};

#[derive(
    AsRef,
    Debug,
    Copy,
    Clone,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    Display,
    Into,
    From,
    FromStr,
    Serialize,
    Deserialize,
)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct BreakoutId(u32);
