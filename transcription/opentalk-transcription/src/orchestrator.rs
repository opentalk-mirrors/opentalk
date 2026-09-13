// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use opentalk_orchestrator_client::{
    Metrics, RegisterTranscription, RegisterType, ServiceResource, TranscriptionResource,
    client::StateProvider,
};
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::TranscriptionTasks;

/// Reports the service's running transcriptions and load to the orchestrator.
pub(crate) struct OrchestratorStateProvider {
    tasks: TranscriptionTasks,
    system: System,
}

impl OrchestratorStateProvider {
    /// Creates a new state provider backed by the shared set of transcription tasks.
    pub fn new(tasks: TranscriptionTasks) -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
        );
        system.refresh_cpu_usage();

        Self { tasks, system }
    }

    /// Returns the current global CPU usage as a percentage clamped to `0..=100`.
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

    async fn on_resource_collision(&mut self, resources: &[ServiceResource]) -> anyhow::Result<()> {
        let colliding_resources = resources
            .iter()
            .filter_map(|resource| match resource {
                ServiceResource::Transcription(transcription_resource) => {
                    Some(*transcription_resource)
                }
                other => {
                    log::error!("Received invalid conflicting resource from orchestrator: {other}");
                    None
                }
            })
            .collect::<Vec<_>>();

        log::warn!("Orchestrator reported resource collision for rooms: {colliding_resources:?}");

        let mut tasks = self.tasks.lock().await;

        tasks.retain(|target, handle| {
            let resource = TranscriptionResource {
                room_id: target.room_id,
                breakout_id: target.breakout_room,
            };

            if colliding_resources.contains(&resource) {
                log::info!("Stopping transcription task for {target:?}");
                handle.exit_task();
                false
            } else {
                true
            }
        });

        Ok(())
    }
}
