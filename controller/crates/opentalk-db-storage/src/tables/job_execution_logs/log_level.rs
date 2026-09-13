// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use derive_more::Display;
use opentalk_inventory as inventory;
use opentalk_types_common::sql_enum;

sql_enum!(
    #[derive(PartialEq, Eq, Display)]
    LogLevel,
    "log_level",
    LogLevelType,
    {
        Trace= b"trace",
        Debug = b"debug",
        Info = b"info",
        Warn = b"warn",
        Error = b"error",
    }
);

impl From<LogLevel> for inventory::JobExecutionLogLevel {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => Self::Trace,
            LogLevel::Debug => Self::Debug,
            LogLevel::Info => Self::Info,
            LogLevel::Warn => Self::Warn,
            LogLevel::Error => Self::Error,
        }
    }
}

impl From<inventory::JobExecutionLogLevel> for LogLevel {
    fn from(value: inventory::JobExecutionLogLevel) -> Self {
        use inventory::JobExecutionLogLevel as Other;
        match value {
            Other::Trace => Self::Trace,
            Other::Debug => Self::Debug,
            Other::Info => Self::Info,
            Other::Warn => Self::Warn,
            Other::Error => Self::Error,
        }
    }
}
