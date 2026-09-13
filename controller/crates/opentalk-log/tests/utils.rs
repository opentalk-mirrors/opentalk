// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(dead_code)]

use std::sync::RwLock;

#[derive(Debug)]
pub struct DummyLogger {
    entries: RwLock<Vec<(log::Level, String)>>,
}

impl DummyLogger {
    pub const fn new() -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
        }
    }

    pub fn entries(&self) -> Vec<(log::Level, String)> {
        self.entries.read().unwrap().clone()
    }

    pub fn clear(&self) {
        self.entries.write().unwrap().clear();
    }
}

impl Default for DummyLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl log::Log for DummyLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        println!("LOGGING: {record:#?}");
        self.entries
            .write()
            .unwrap()
            .push((record.level(), record.args().to_string()));
    }

    fn flush(&self) {}
}
