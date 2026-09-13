// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// Environment variable used to configure the log filter, taking precedence over `RUST_LOG`.
const LOG_ENV: &str = "TRANSCRIPTION_LOG";

/// Default log directives, applied when neither [`LOG_ENV`] nor `RUST_LOG` is set.
const DEFAULT_LOGGING_DIRECTIVES: &str = "warn,opentalk_transcription=info";

/// Initializes the global logger.
///
/// The log filter directives are resolved with the following precedence:
/// `TRANSCRIPTION_LOG` > `RUST_LOG` > [`DEFAULT_LOGGING_DIRECTIVES`].
pub(crate) fn init() {
    let directives = read_env_var(LOG_ENV)
        .or_else(|| read_env_var("RUST_LOG"))
        .unwrap_or_else(|| DEFAULT_LOGGING_DIRECTIVES.to_owned());

    env_logger::Builder::new().parse_filters(&directives).init();
}

/// Reads an environment variable, treating an empty value the same as an unset one.
fn read_env_var(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|value| !value.is_empty())
}
