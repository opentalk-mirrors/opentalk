// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{sync::atomic::Ordering, time::Duration};

use anyhow::Result;
use sysinfo::{self, RefreshKind, System};

use super::IS_FEASIBLE;
use crate::system_info::CURRENT_LOAD;

pub(crate) fn cpu_usage_poll(cutoff: u8) -> Result<()> {
    const INTERVAL: Duration = Duration::from_secs(1u64);
    let mut rti = RuntimeInformation::new(cutoff);
    loop {
        rti.setup_cpu_poll();
        std::thread::sleep(INTERVAL);
    }
}

struct RuntimeInformation {
    cutoff: u8,
    system: System,
}

impl RuntimeInformation {
    pub(crate) fn new(cutoff: u8) -> Self {
        Self {
            cutoff,
            system: System::new_with_specifics(RefreshKind::everything()),
        }
    }

    pub(crate) fn setup_cpu_poll(&mut self) {
        // Enforce an update happened
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.system.refresh_cpu_all();

        let cores = self.system.cpus().len();
        let max_cpu_usage: u32 = u32::from(self.cutoff) * cores as u32;

        let mut cpu_usage = 0;
        for cpu in self.system.cpus() {
            cpu_usage += cpu.cpu_usage() as u32;
        }

        IS_FEASIBLE.store(cpu_usage <= max_cpu_usage, Ordering::Relaxed);
        CURRENT_LOAD.store((cpu_usage / cores as u32) as u8, Ordering::Relaxed);
    }
}
