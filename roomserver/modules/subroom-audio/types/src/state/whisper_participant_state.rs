// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
use serde::{Deserialize, Serialize};
// use strum::{AsRefStr, Display, EnumCount, EnumIter, EnumString, IntoStaticStr, VariantNames};

/// The invite state for a whisper participant
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
// #[strum(serialize_all = "snake_case")]
pub enum WhisperState {
    /// The creator of the whisper group
    Creator,
    /// The participant has been invited but did not reply yet
    #[default]
    Invited,
    /// The participant accepted the invite
    Accepted,
}
