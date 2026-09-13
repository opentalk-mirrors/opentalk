// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use egui::KeyboardShortcut;
use opentalk_roomserver_client::api::event::SignalingEvent;

use crate::settings::DuiSettings;

pub enum Received {
    SignalingEvent(Box<SignalingEvent>),
    Invalid,
}

impl Received {
    /// Returns `true` if the received is [`Invalid`].
    ///
    /// [`Invalid`]: Received::Invalid
    #[must_use]
    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

impl From<String> for Received {
    fn from(received: String) -> Self {
        if let Ok(event) = serde_json::from_str(&received) {
            Self::SignalingEvent(event)
        } else {
            Self::Invalid
        }
    }
}

pub trait SignalingPlugin: std::fmt::Debug {
    fn name(&self) -> &str;
    fn shortcut(&self) -> Option<&KeyboardShortcut> {
        None
    }

    fn handle_events(&mut self, settings: &mut DuiSettings, received: &[Received]) -> Vec<String>;

    fn ui(&mut self, ui: &mut egui::Ui, settings: &mut DuiSettings) -> Vec<String>;
}
