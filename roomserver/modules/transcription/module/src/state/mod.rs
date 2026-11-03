// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use automatic_transcription::AutomaticTranscription;
use opentalk_roomserver_signaling::module_context::ModuleContext;
use opentalk_roomserver_types::signaling::module_error::SignalingModuleError;
use opentalk_roomserver_types_transcription::{
    config::AutomaticTranscriptionConfig,
    event::TranscriptionError,
    status::{TranscriptionModes, TranscriptionStatus},
};
use opentalk_types_common::{time::TimeZone, transcription::RequestId};
use opentalk_types_signaling::ParticipantId;

use crate::{TranscriptionModule, state::automatic_transcription::FinishedAutomaticTranscription};

pub mod automatic_transcription;

#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum ServiceAction {
    Start {
        request_id: RequestId,
        language: Option<String>,
    },
    /// Ask the running service to stop. Participants are notified once it reports `Stopped`.
    Stop,
    /// Withdraw a requested service that never reported `Started`. It will not confirm anything.
    Cancel,
    None,
}

/// Internal per-room transcription state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptionRoomState {
    /// The current lifecycle of the transcription service.
    pub service: ServiceState,
    /// The language of the transcription service.
    pub language: Option<String>,
    /// The active transcription modes.
    pub modes: ActiveTranscriptionModes,
}

impl TranscriptionRoomState {
    pub fn reconcile(&mut self) -> ServiceAction {
        match (self.modes.needs_service(), self.service) {
            (true, ServiceState::Stopped | ServiceState::Stopping { participant_id: _ }) => {
                let request_id = RequestId::generate();
                self.service = ServiceState::Requested { request_id };

                ServiceAction::Start {
                    request_id,
                    language: self.language.clone(),
                }
            }
            (
                true,
                ServiceState::Requested { request_id: _ }
                | ServiceState::Running { participant_id: _ },
            ) => ServiceAction::None,
            (false, ServiceState::Stopped | ServiceState::Stopping { participant_id: _ }) => {
                ServiceAction::None
            }
            (false, ServiceState::Running { participant_id }) => {
                self.service = ServiceState::Stopping { participant_id };
                ServiceAction::Stop
            }
            (false, ServiceState::Requested { request_id: _ }) => {
                self.service = ServiceState::Stopped;
                ServiceAction::Cancel
            }
        }
    }

    /// The status of the transcription service
    pub fn status(&self) -> TranscriptionStatus {
        match self.service {
            ServiceState::Stopped | ServiceState::Stopping { participant_id: _ } => {
                TranscriptionStatus::Inactive
            }
            ServiceState::Requested { request_id: _ } => TranscriptionStatus::Requested {
                language: self.language.clone(),
            },
            ServiceState::Running { participant_id: _ } => TranscriptionStatus::Running {
                language: self.language.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceState {
    /// The transcription service is not running
    Stopped,
    /// The transcription service has been requested but is not yet running.
    Requested {
        request_id: RequestId,
    },
    /// The transcription service is running.
    Running {
        participant_id: ParticipantId,
    },
    Stopping {
        participant_id: ParticipantId,
    },
}

impl ServiceState {
    pub fn id(self) -> Option<ParticipantId> {
        match self {
            ServiceState::Stopped | ServiceState::Requested { request_id: _ } => None,
            ServiceState::Running { participant_id }
            | ServiceState::Stopping { participant_id } => Some(participant_id),
        }
    }
}

/// The active transcription modes for a room.
///
/// This can be [`live_transcription`](Self::live_transcription),
/// [`automatic_transcription`](Self::automatic_transcription) or both, but never neither.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveTranscriptionModes {
    live_transcription: bool,
    automatic_transcription: Option<AutomaticTranscription>,
}

impl ActiveTranscriptionModes {
    /// Starts a new transcription with the given `mode`.
    pub fn start(
        ctx: &ModuleContext<'_, TranscriptionModule>,
        mode: TranscriptionModeConfig,
    ) -> Self {
        match mode {
            TranscriptionModeConfig::LiveTranscription => Self {
                live_transcription: true,
                automatic_transcription: None,
            },
            TranscriptionModeConfig::AutomaticTranscription { config, time_zone } => Self {
                live_transcription: false,
                automatic_transcription: Some(AutomaticTranscription::new(ctx, config, time_zone)),
            },
        }
    }

    /// Activates the given `mode`.
    ///
    /// Does not deactivate any other mode.
    pub fn activate(
        &mut self,
        ctx: &ModuleContext<'_, TranscriptionModule>,
        mode: TranscriptionModeConfig,
    ) -> Changed {
        match mode {
            TranscriptionModeConfig::LiveTranscription => {
                if self.live_transcription {
                    return Changed::No;
                }

                self.live_transcription = true;
            }
            TranscriptionModeConfig::AutomaticTranscription { config, time_zone } => {
                if self.automatic_transcription.is_some() {
                    return Changed::No;
                }

                self.automatic_transcription =
                    Some(AutomaticTranscription::new(ctx, config, time_zone));
            }
        }

        Changed::Yes
    }

    pub fn pause_automatic_transcription(&mut self) -> Result<Changed, NotActive> {
        self.automatic_transcription
            .as_mut()
            .map(|p| if p.pause() { Changed::Yes } else { Changed::No })
            .ok_or(NotActive)
    }

    pub fn resume_automatic_transcription(&mut self) -> Result<Changed, NotActive> {
        self.automatic_transcription
            .as_mut()
            .map(|p| {
                if p.resume() {
                    Changed::Yes
                } else {
                    Changed::No
                }
            })
            .ok_or(NotActive)
    }

    /// Returns `true` if live transcription is active.
    #[must_use]
    pub fn live_transcription(&self) -> bool {
        self.live_transcription
    }

    pub fn deactivate_live_transcription(&mut self) {
        self.live_transcription = false;
    }

    /// Returns a reference to the automatic transcription, if it is active.
    #[must_use]
    pub fn automatic_transcription(&self) -> Option<&AutomaticTranscription> {
        self.automatic_transcription.as_ref()
    }

    pub fn deactivate_automatic_transcription(
        &mut self,
        ctx: &ModuleContext<'_, TranscriptionModule>,
    ) -> Option<FinishedAutomaticTranscription> {
        self.automatic_transcription
            .take()
            .map(|automatic_transcription| automatic_transcription.stop(ctx))
    }

    /// Returns a mutable reference to the automatic transcription, if it is active.
    #[must_use]
    pub fn automatic_transcription_mut(&mut self) -> Option<&mut AutomaticTranscription> {
        self.automatic_transcription.as_mut()
    }

    #[must_use]
    pub fn needs_service(&self) -> bool {
        self.live_transcription
            || self
                .automatic_transcription
                .as_ref()
                .is_some_and(|automatic_transcription| !automatic_transcription.paused)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.live_transcription && self.automatic_transcription.is_none()
    }
}

impl From<&ActiveTranscriptionModes> for TranscriptionModes {
    fn from(value: &ActiveTranscriptionModes) -> Self {
        Self {
            live_transcription: value.live_transcription,
            automatic_transcription: value.automatic_transcription.as_ref().map(Into::into),
        }
    }
}

/// The configuration for a transcription mode.
#[derive(Debug, Clone, Copy)]
pub enum TranscriptionModeConfig {
    LiveTranscription,
    AutomaticTranscription {
        config: AutomaticTranscriptionConfig,
        time_zone: TimeZone,
    },
}

impl TranscriptionModeConfig {
    /// Returns the [`TranscriptionMode`] this config targets.
    pub fn mode(&self) -> TranscriptionMode {
        match self {
            TranscriptionModeConfig::LiveTranscription => TranscriptionMode::LiveTranscription,
            TranscriptionModeConfig::AutomaticTranscription { .. } => {
                TranscriptionMode::AutomaticTranscription
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TranscriptionMode {
    LiveTranscription,
    AutomaticTranscription,
}

#[derive(Debug, Clone, Copy)]
#[must_use]
pub enum Changed {
    Yes,
    No,
}

impl Changed {
    pub fn is_unchanged(self) -> bool {
        match self {
            Changed::Yes => false,
            Changed::No => true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NotActive;

impl From<NotActive> for SignalingModuleError<TranscriptionError> {
    fn from(_: NotActive) -> Self {
        TranscriptionError::NotActive.into()
    }
}
