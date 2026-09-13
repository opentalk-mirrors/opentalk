// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Setting configuration used in the signaling process for the `automod` namespace

mod parameter;
mod selection_strategy;

use opentalk_types_signaling::ParticipantId;
pub use parameter::Parameter;
pub use selection_strategy::SelectionStrategy;
use serde::{Deserialize, Serialize};

/// Used to communicate with the frontend
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendConfig {
    /// Parameters that describe the configuration of the automod session
    #[serde(flatten)]
    pub parameter: Parameter,

    /// See documentation of [`super::event::AutomodEvent::SpeakerUpdated`]
    pub history: Vec<ParticipantId>,

    /// See documentation of [`super::event::AutomodEvent::SpeakerUpdated`]
    pub remaining: Vec<ParticipantId>,

    /// The ID of the participant who started the automoderation session
    pub issued_by: ParticipantId,
}

impl FrontendConfig {
    /// Converts the config into a public config, which is modified to not show the list of
    /// available participants if configured.
    pub fn into_public(mut self) -> PublicConfig {
        let hide_list_if_requested = matches!(
            self.parameter.selection_strategy,
            SelectionStrategy::Playlist | SelectionStrategy::Random
        );

        if hide_list_if_requested && !self.parameter.show_remaining {
            self.remaining.clear();
        }

        PublicConfig(self)
    }
}

/// Typed version of the frontend-config that will be sent to the frontend, may only be created
/// using [`FrontendConfig::into_public`]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicConfig(FrontendConfig);
