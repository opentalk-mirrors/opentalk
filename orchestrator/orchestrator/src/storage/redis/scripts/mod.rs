// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Contains all redis lua scripts to manage the state of a orchestator cluster. Each script is in
//! its own module and can be used by the orchestrator storage implementation.

mod add_instance;
mod cleanup;
mod dead_orchestrators;
mod get_by_resource;
mod init;
mod remove_instance;
mod select_instance;
mod service_metrics;

pub(crate) use add_instance::{AddInstanceScriptError, add_instance};
pub(crate) use cleanup::cleanup;
pub(crate) use dead_orchestrators::get_dead_orchestrators;
pub(crate) use get_by_resource::get_instance_by_resource;
pub(crate) use init::init;
pub(crate) use remove_instance::remove_instance;
pub(crate) use select_instance::select_instance;
pub(crate) use service_metrics::get_service_metrics;
