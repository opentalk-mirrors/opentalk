// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Database implementation of the data storage facade.

mod asset;
mod authorization;
mod database_connection;
mod database_connection_pool;
mod error;
mod event;
mod event_invite;
mod event_shared_folder;
mod event_training_participation_report;
mod group;
mod job_execution;
mod module_resource;
mod room;
mod room_invite;
mod room_sip_config;
mod room_streaming_target;
mod tariff;
mod tenant;
mod transaction_manager;
mod user;
mod utils;

pub use database_connection::DatabaseConnection;
pub use database_connection_pool::DatabaseConnectionPool;
use opentalk_inventory::{Error, Result};
