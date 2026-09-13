// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#[path = "utils.rs"]
mod utils;

use log::Level;
use opentalk_log::{debug, error, info, log, trace, warn};
use utils::DummyLogger;

#[test]
fn log_general() {
    let dummy = DummyLogger::new();

    log!(log: &dummy, Level::Warn, "Hello, {}", "world");

    assert_eq!(
        vec![(Level::Warn, String::from("Hello, world"))],
        dummy.entries()
    );
}

#[test]
fn log_multiple() {
    let dummy = DummyLogger::new();

    error!(log: &dummy, "Hello, {}", "error");
    warn!(log: &dummy, "Hello, {}", "warn");
    info!(log: &dummy, "Hello, {}", "info");
    debug!(log: &dummy, "Hello, {}", "debug");
    trace!(log: &dummy, "Hello, {}", "trace");

    assert_eq!(
        vec![
            (Level::Error, String::from("Hello, error")),
            (Level::Warn, String::from("Hello, warn")),
            (Level::Info, String::from("Hello, info")),
            (Level::Debug, String::from("Hello, debug")),
            (Level::Trace, String::from("Hello, trace"))
        ],
        dummy.entries()
    );
}

#[test]
fn log_to_different_loggers() {
    let dummy_a = DummyLogger::new();
    let dummy_b = DummyLogger::new();

    error!(log: &dummy_a, "Hello, {}", "error");
    warn!(log: &dummy_b, "Hello, {}", "warn");
    info!(log: &dummy_a, "Hello, {}", "info");
    debug!(log: &dummy_b, "Hello, {}", "debug");
    trace!(log: &dummy_a, "Hello, {}", "trace");

    assert_eq!(
        vec![
            (Level::Error, String::from("Hello, error")),
            (Level::Info, String::from("Hello, info")),
            (Level::Trace, String::from("Hello, trace"))
        ],
        dummy_a.entries()
    );
    assert_eq!(
        vec![
            (Level::Warn, String::from("Hello, warn")),
            (Level::Debug, String::from("Hello, debug")),
        ],
        dummy_b.entries()
    );
}

#[test]
fn log_error() {
    let dummy = DummyLogger::new();
    error!(log: &dummy, "Hello, {}", "error");

    assert_eq!(
        vec![(Level::Error, String::from("Hello, error"))],
        dummy.entries()
    );
}

#[test]
fn log_warn() {
    let dummy = DummyLogger::new();
    warn!(log: &dummy, "Hello, {}", "warn");

    assert_eq!(
        vec![(Level::Warn, String::from("Hello, warn"))],
        dummy.entries()
    );
}

#[test]
fn log_info() {
    let dummy = DummyLogger::new();
    info!(log: &dummy, "Hello, {}", "info");

    assert_eq!(
        vec![(Level::Info, String::from("Hello, info"))],
        dummy.entries()
    );
}

#[test]
fn log_debug() {
    let dummy = DummyLogger::new();
    debug!(log: &dummy, "Hello, {}", "debug");

    assert_eq!(
        vec![(Level::Debug, String::from("Hello, debug"))],
        dummy.entries()
    );
}

#[test]
fn log_trace() {
    let dummy = DummyLogger::new();
    trace!(log: &dummy, "Hello, {}", "trace");

    assert_eq!(
        vec![(Level::Trace, String::from("Hello, trace"))],
        dummy.entries()
    );
}
