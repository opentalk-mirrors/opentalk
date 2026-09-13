// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! OpenTalk Controller service
//!
//! This crate contains the default OpenTalk Controller backend implementation.

pub mod controller_backend;
pub mod events;
pub mod helpers;
pub mod metrics;
pub mod oidc;
pub mod phone_numbers;
mod redis_wrapper;
pub mod services;

pub mod user_profiles;

pub use controller_backend::ControllerBackend;
pub use helpers::{ToUserProfile, email_to_libravatar_url};
pub use redis_wrapper::{RedisConnection, RedisMetrics};
use snafu::{Backtrace, Snafu};

type Result<T, E = Whatever> = std::result::Result<T, E>;

/// Send and Sync variant of [`snafu::Whatever`]
#[derive(Debug, Snafu)]
#[snafu(whatever)]
#[snafu(display("{message}"))]
pub struct Whatever {
    #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, Some)))]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
    message: String,
    backtrace: Backtrace,
}
