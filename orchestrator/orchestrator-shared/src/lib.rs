// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::collections::HashMap;

use opentalk_types_common::rooms::RoomId;
use serde::{Deserialize, Serialize};
pub use url::Url;

use crate::services::ServiceState;
pub use crate::{
    event::Event,
    register::{Register, RegisterData, RegisterResponse, RegisterType, ServiceAddress},
    services::{
        ServiceResource,
        recorder::{RecorderEvent, RecorderResource, RegisterRecorder},
        roomserver::{RegisterRoomserver, RoomserverEvent},
        transcription::{RegisterTranscription, TranscriptionEvent, TranscriptionResource},
    },
};

pub mod error;
mod event;
mod register;
pub mod services;

#[derive(Debug, Clone, Copy)]
pub enum ServiceKind {
    Recorder,
    Roomserver,
    Transcription,
}

impl std::fmt::Display for ServiceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceKind::Recorder => write!(f, "recorder"),
            ServiceKind::Roomserver => write!(f, "roomserver"),
            ServiceKind::Transcription => write!(f, "transcription"),
        }
    }
}

impl From<&RegisterType> for ServiceKind {
    fn from(register_type: &RegisterType) -> Self {
        match register_type {
            RegisterType::Recorder(_) => ServiceKind::Recorder,
            RegisterType::Roomserver(_) => ServiceKind::Roomserver,
            RegisterType::Transcription(_) => ServiceKind::Transcription,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrchestratorMetrics {
    pub roomservers: HashMap<Url, ServiceState<RoomId>>,
    pub recorders: HashMap<Url, ServiceState<RecorderResource>>,
    pub transcription: HashMap<Url, ServiceState<TranscriptionResource>>,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Metrics {
    pub load: u8,
    pub accepting_jobs: bool,
}
