// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_orchestrator_client::{
    Metrics, RegisterTranscription, RegisterType, TranscriptionResource, client::StateProvider,
};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::TranscriptionTasks;

pub(crate) struct OrchestratorStateProvider {
    tasks: TranscriptionTasks,
    system: System,
}

impl OrchestratorStateProvider {
    pub fn new(tasks: TranscriptionTasks) -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
        );
        system.refresh_cpu_usage();

        Self { tasks, system }
    }

    fn calculate_load(&mut self) -> u8 {
        self.system.refresh_cpu_usage();
        let cpu_usage = self.system.global_cpu_usage();

        (cpu_usage as u8).min(100)
    }
}

#[async_trait::async_trait]
impl StateProvider for OrchestratorStateProvider {
    async fn register_type(&mut self) -> RegisterType {
        let tasks = self.tasks.lock().await;

        let rooms = tasks
            .keys()
            .cloned()
            .map(|target| TranscriptionResource {
                room_id: target.room_id,
                breakout_id: target.breakout_room,
            })
            .collect();

        RegisterType::Transcription(RegisterTranscription { rooms })
    }

    async fn metrics(&mut self) -> Metrics {
        Metrics {
            load: self.calculate_load(),
            accepting_jobs: true,
        }
    }
}
