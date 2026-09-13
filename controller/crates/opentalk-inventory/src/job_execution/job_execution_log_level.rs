// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

/// The status of a maintenance job.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    strum::AsRefStr,
    strum::Display,
    strum::EnumCount,
    strum::EnumIter,
    strum::EnumString,
    strum::VariantNames,
    strum::IntoStaticStr,
    clap::ValueEnum,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum JobExecutionLogLevel {
    /// TRACE log level.
    Trace,

    /// DEBUG log level.
    Debug,

    /// INFO log level.
    Info,

    /// WARN log level.
    Warn,

    /// ERROR log level.
    Error,
}
