// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

pub(crate) mod cpu;
pub(crate) mod gpu_intel;

pub(crate) static IS_FEASIBLE: AtomicBool = AtomicBool::new(true);
pub(crate) static CURRENT_LOAD: AtomicU8 = AtomicU8::new(0);

pub(crate) fn is_new_recording_feasible() -> bool {
    IS_FEASIBLE.load(Ordering::Relaxed)
}
