// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use serde::{Deserialize, Serialize};

use crate::event::stop_kind::StopKind;

/// The current timer has been stopped
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stopped {
    /// The stop kind
    #[serde(flatten)]
    pub kind: StopKind,
    /// An optional reason to all participants. Set by moderator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
