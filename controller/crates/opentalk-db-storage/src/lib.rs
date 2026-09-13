// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::extra_unused_lifetimes)]

//! Contains the database ORM and database migrations for the controller/storage
//! Builds upon opentalk-database

#[macro_use]
extern crate diesel;

// postgres functions
use diesel::sql_types::Text;

#[macro_use]
mod macros;
mod schema;

pub mod migrations;
pub mod paginate;
pub mod paginated;
pub mod queries;
pub mod tables;
pub mod utils;

define_sql_function!(fn lower(x: Text) -> Text);
define_sql_function!(fn levenshtein(x: Text, y: Text) -> Integer);
define_sql_function!(fn soundex(x: Text) -> Text);

// SQL types reexport for schema.rs
pub mod sql_types {

    pub use diesel::sql_types::*;
    pub use opentalk_types_common::{
        events::invites::{
            EmailInviteRoleType as EmailInviteRole, EventInviteStatusType as EventInviteStatus,
            InviteRoleType as InviteRole,
        },
        rooms::GuestAccessType as GuestAccess,
        streaming::StreamingKindType as StreamingKind,
        tariffs::TariffStatusType as TariffStatus,
        users::ThemeType as Theme,
    };

    pub use super::tables::{
        event_exceptions::EventExceptionKindType as EventExceptionKind,
        job_execution_logs::LogLevelType as LogLevel, job_executions::JobStatusType as JobStatus,
        jobs::JobTypeType as JobType,
    };
}
