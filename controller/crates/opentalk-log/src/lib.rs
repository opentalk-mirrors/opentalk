// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Logging helper functionality for OpenTalk

#![warn(
    bad_style,
    missing_debug_implementations,
    missing_docs,
    overflowing_literals,
    patterns_in_fns_without_body,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]

use std::panic::Location;

use log::Log;

// WARNING: this is not part of the crate's public API and is subject to change at any time
// Copied over from the `log` crate, but extended by the `logger` argument
#[doc(hidden)]
pub fn __private_api_log(
    logger: &dyn Log,
    args: std::fmt::Arguments,
    level: log::Level,
    &(target, module_path, location): &(&str, &'static str, &'static Location),
) {
    logger.log(
        &log::Record::builder()
            .args(args)
            .level(level)
            .target(target)
            .module_path_static(Some(module_path))
            .file_static(Some(location.file()))
            .line(Some(location.line()))
            .build(),
    );
}

/// Standard logging macro.
///
/// In contrast to the `log::log` macro, this has a variant that accepts
/// a reference to an additional instance of any type that implements the
/// `log::Log` trait.
///
/// # Examples
///
/// ```edition2018
/// use opentalk_log::log;
/// use log::Level;
///
/// # fn main() {
/// let data = (42, "Forty-two");
/// let private_data = "private";
///
/// log!(Level::Error, "Received errors: {}, {}", data.0, data.1);
/// log!(target: "app_events", Level::Warn, "App warning: {}, {}, {}",
///     data.0, data.1, private_data);
///
/// struct DummyLogger;
///
/// impl log::Log for DummyLogger {
///     fn enabled(&self, metadata: &log::Metadata) -> bool {
///         // Might be any useful information, this is just for demonstration purposes
///         log::logger().enabled(metadata)
///     }
///
///     fn log(&self, record: &log::Record) {
///         // Might be any useful information, this is just for demonstration purposes
///         log::logger().log(record);
///     }
///
///     fn flush(&self) {
///         // Might be any useful information, this is just for demonstration purposes
///         log::logger().flush();
///     }
/// }
///
///
///
/// # }
/// ```
#[macro_export]
macro_rules! log {
    // log!(target: "my_target", log: log::logger(), Level::Info, "a {} event", "log");
    (target: $target:expr, log: $logger:expr, $lvl:expr, $($arg:tt)+) => ({
        let logger = $logger;
        let lvl = $lvl;
        if lvl <= ::log::STATIC_MAX_LEVEL {
            $crate::__private_api_log(
                logger,
                ::log::__private_api::format_args!($($arg)+),
                lvl,
                &($target, ::log::__private_api::module_path!(), ::log::__private_api::loc()),
            );
        }
    });

    // log!(target: "my_target", Level::Info, "a {} event", "log");
    (target: $target:expr, $lvl:expr, $($arg:tt)+) => ($crate::log!(target: ::log::__private_api::module_path!(), log: ::log::logger(), $lvl,  $($arg)+));

    // log!(log: log::logger(), Level::Info, "a log event")
    (log: $logger:expr, $lvl:expr, $($arg:tt)+) => ($crate::log!(target: ::log::__private_api::module_path!(), log: $logger, $lvl, $($arg)+));

    // log!(Level::Info, "a log event")
    ($lvl:expr, $($arg:tt)+) => ($crate::log!(target: ::log::__private_api::module_path!(), log: ::log::logger(), $lvl, $($arg)+));
}

/// Log a message at the error level.
///
/// In contrast to the `log::error` macro, this has a variant that accepts
/// a reference to an additional instance of any type that implements the
/// `log::Log` trait.
#[macro_export]
macro_rules! error {
    // error!(log: log::logger(), "a {} event", "log")
    (log: $logger:expr, $($arg:tt)+) => ($crate::log!(log: $logger, ::log::Level::Error, $($arg)+));

    // error!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!(::log::Level::Error, $($arg)+));
}

/// Log a message at the warn level.
///
/// In contrast to the `log::warn` macro, this has a variant that accepts
/// a reference to an additional instance of any type that implements the
/// `log::Log` trait.
#[macro_export]
macro_rules! warn {
    // warn!(log: log::logger(), "a {} event", "log")
    (log: $logger:expr, $($arg:tt)+) => ($crate::log!(log: $logger, ::log::Level::Warn, $($arg)+));

    // warn!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!(::log::Level::Warn, $($arg)+));
}

/// Log a message at the info level.
///
/// In contrast to the `log::info` macro, this has a variant that accepts
/// a reference to an additional instance of any type that implements the
/// `log::Log` trait.
#[macro_export]
macro_rules! info {
    // info!(log: log::logger(), "a {} event", "log")
    (log: $logger:expr, $($arg:tt)+) => ($crate::log!(log: $logger, ::log::Level::Info, $($arg)+));

    // info!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!(::log::Level::Info, $($arg)+));
}

/// Log a message at the debug level.
///
/// In contrast to the `log::debug` macro, this has a variant that accepts
/// a reference to an additional instance of any type that implements the
/// `log::Log` trait.
#[macro_export]
macro_rules! debug {
    // debug!(log: log::logger(), "a {} event", "log")
    (log: $logger:expr, $($arg:tt)+) => ($crate::log!(log: $logger, ::log::Level::Debug, $($arg)+));

    // debug!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!(::log::Level::Debug, $($arg)+));
}

/// Log a message at the trace level.
///
/// In contrast to the `log::trace` macro, this has a variant that accepts
/// a reference to an additional instance of any type that implements the
/// `log::Log` trait.
#[macro_export]
macro_rules! trace {
    // trace!(log: log::logger(), "a {} event", "log")
    (log: $logger:expr, $($arg:tt)+) => ($crate::log!(log: $logger, ::log::Level::Trace, $($arg)+));

    // trace!("a {} event", "log")
    ($($arg:tt)+) => ($crate::log!(::log::Level::Trace, $($arg)+));
}
