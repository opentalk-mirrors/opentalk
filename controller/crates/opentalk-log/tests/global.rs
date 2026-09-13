// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Tests that verify whether the global logger configured by `log::set_logger`
//! is used by the macros from this crate.

#[path = "utils.rs"]
mod utils;

use std::sync::OnceLock;

use log::Level;
use opentalk_log::{debug, error, info, log, trace, warn};
use serial_test::serial;
use utils::DummyLogger;

static LOGGER: OnceLock<DummyLogger> = OnceLock::new();

#[test]
#[serial]
fn serial_test_log_general() {
    initialize_logging();

    log!(Level::Warn, "Hello, {}", "world");

    assert_eq!(
        vec![(Level::Warn, String::from("Hello, world"))],
        dummy().entries()
    );
}

#[test]
#[serial]
fn serial_test_log_multiple() {
    initialize_logging();

    error!("Hello, {}", "error");
    warn!("Hello, {}", "warn");
    info!("Hello, {}", "info");
    debug!("Hello, {}", "debug");
    trace!("Hello, {}", "trace");

    assert_eq!(
        vec![
            (Level::Error, String::from("Hello, error")),
            (Level::Warn, String::from("Hello, warn")),
            (Level::Info, String::from("Hello, info")),
            (Level::Debug, String::from("Hello, debug")),
            (Level::Trace, String::from("Hello, trace"))
        ],
        dummy().entries()
    );
}

#[test]
#[serial]
fn serial_test_log_error() {
    initialize_logging();

    error!("Hello, {}", "error");

    assert_eq!(
        vec![(Level::Error, String::from("Hello, error"))],
        dummy().entries()
    );
}

#[test]
#[serial]
fn serial_test_log_warn() {
    initialize_logging();

    warn!("Hello, {}", "warn");

    assert_eq!(
        vec![(Level::Warn, String::from("Hello, warn"))],
        dummy().entries()
    );
}

#[test]
#[serial]
fn serial_test_log_info() {
    initialize_logging();

    info!("Hello, {}", "info");

    assert_eq!(
        vec![(Level::Info, String::from("Hello, info"))],
        dummy().entries()
    );
}

#[test]
#[serial]
fn serial_test_log_debug() {
    initialize_logging();

    debug!("Hello, {}", "debug");

    assert_eq!(
        vec![(Level::Debug, String::from("Hello, debug"))],
        dummy().entries()
    );
}

#[test]
#[serial]
fn serial_test_log_trace() {
    initialize_logging();

    trace!("Hello, {}", "trace");

    assert_eq!(
        vec![(Level::Trace, String::from("Hello, trace"))],
        dummy().entries()
    );
}

fn initialize_logging() {
    match LOGGER.get() {
        Some(l) => l.clear(),
        None => {
            LOGGER.set(DummyLogger::new()).unwrap();
            log::set_logger(LOGGER.get().unwrap()).expect("Could not set logger, already set");
        }
    }
}

fn dummy() -> &'static DummyLogger {
    LOGGER.get().expect("Logger not initialized")
}
