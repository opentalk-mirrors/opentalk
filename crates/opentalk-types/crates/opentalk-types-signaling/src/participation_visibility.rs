// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Indicates the visibility of each Participant
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticipationVisibility {
    /// Indicates, that the participant is visible in the conference
    Visible,
    /// Indicates, that the participant is hidden from the conference
    Hidden,
}

impl ParticipationVisibility {
    /// Checks if the visibility is set to Visible
    pub const fn is_visible(&self) -> bool {
        matches!(self, ParticipationVisibility::Visible)
    }

    /// Checks if the visibility is set to Hidden
    pub const fn is_hidden(&self) -> bool {
        matches!(self, ParticipationVisibility::Hidden)
    }
}
