// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::ParticipantId;

/// Status information about a participant
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Participant {
    /// The id of the participant
    pub id: ParticipantId,

    /// Module data for the participant
    #[cfg(feature = "serde")]
    #[serde(flatten)]
    pub module_data: crate::ModulePeerData,
}

impl Participant {
    /// Gets the inner module data of a Participant
    #[cfg(feature = "serde")]
    pub fn get_module<T: crate::SignalingModulePeerFrontendData>(
        &self,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.get::<T>()
    }

    /// Updates the inner module data of a Participant and returns the new data
    #[cfg(feature = "serde")]
    pub fn update_module<T: crate::SignalingModulePeerFrontendData, F: FnOnce(&mut T)>(
        &mut self,
        update: F,
    ) -> Result<Option<T>, serde_json::Error> {
        self.module_data.update::<T, F>(update)
    }
}
