// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::any::Any;

use opentalk_types_common::modules::ModuleId;

pub type ResultCallback = Box<dyn FnOnce(Box<dyn Any + Send + 'static>) + Send + 'static>;

/// Holds a command and corresponding metadata send between
/// [`SignalingModule`](crate::signaling_module::SignalingModule)s.
#[derive(Debug)]
pub struct InterModuleMessage {
    /// The module that sent the command
    pub sender: ModuleId,
    /// The module that will receive the command
    pub receiver: ModuleId,
    /// The command for the [`receiver`](Self::receiver). This will be converted to
    /// the corresponding
    /// [`SignalingModule::Internal`](crate::signaling_module::SignalingModule::Internal)
    /// type.
    pub command: Box<dyn Any + Send + 'static>,
}
