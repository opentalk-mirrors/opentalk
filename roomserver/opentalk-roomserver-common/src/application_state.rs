// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

/// The state of the application. This is used to signal all components of the
/// app that the app should shutdown.
#[derive(Debug, Clone, Copy, Default)]
pub enum ApplicationState {
    /// The application is running and should continue doing so.
    #[default]
    Running,

    /// The application is shutting down.
    ShuttingDown,
}

impl ApplicationState {
    /// Returns `true` if the application state is [`ShuttingDown`].
    ///
    /// [`ShuttingDown`]: ApplicationState::ShuttingDown
    pub fn is_shutting_down(&self) -> bool {
        matches!(self, Self::ShuttingDown)
    }
}
