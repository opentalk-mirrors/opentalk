// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use clap::Parser;
use nix::sys::signal::{SIGHUP, kill};
use snafu::{OptionExt, ResultExt};
use sysinfo::{self, Pid, Process, ProcessRefreshKind, RefreshKind, System, get_current_pid};

use crate::Result;

#[derive(Debug, Clone, Parser)]
pub struct Command;

impl Command {
    /// Sends SIGHUP to all process with a different pid and the same name
    pub fn exec(self) -> Result<()> {
        let target_processes = find_target_processes()?;
        if target_processes.is_empty() {
            println!("There is currently no other controller process running");
            Ok(())
        } else {
            send_sighup_to_processes(target_processes)
        }
    }
}

/// Looks for a process with the same name and a different pid, returns [`Option<Pid>`]
fn find_target_processes() -> Result<Vec<Pid>> {
    let system = System::new_with_specifics(
        RefreshKind::default().with_processes(ProcessRefreshKind::everything()),
    );
    let current_process = get_current_process(&system)?;
    Ok(search_for_target_processes(&system, current_process))
}

/// Send SIGHUP to each process in [`Vec<Pid>`]
fn send_sighup_to_processes(processes: Vec<Pid>) -> Result<()> {
    processes
        .into_iter()
        .try_for_each(|pid| kill(nix::unistd::Pid::from_raw(pid.as_u32() as i32), SIGHUP))
        .whatever_context("Failed to get PIDs")
}

/// Returns the [`Process`] of the current running application
fn get_current_process(system: &System) -> Result<&Process> {
    let pid = get_current_pid().whatever_context("Failed to get current process PID")?;
    system
        .process(pid)
        .whatever_context("Failed to get current process, that was not to expect")
}

/// Iterates over all processes to find processes with same name as the current process and a different pid
fn search_for_target_processes(system: &System, current_process: &Process) -> Vec<Pid> {
    system
        .processes()
        .iter()
        .filter(|(pid, process)| {
            **pid != current_process.pid() && process.name() == current_process.name()
        })
        .map(|(pid, _process)| *pid)
        .collect()
}
