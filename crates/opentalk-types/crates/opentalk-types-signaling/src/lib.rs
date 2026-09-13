// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Signaling data types for OpenTalk.

#![deny(
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

mod associated_participant;
mod leave_reason;
#[cfg(feature = "serde")]
mod module_data;
#[cfg(feature = "serde")]
mod module_peer_data;
mod namespaced_command;
mod namespaced_event;
mod participant;
mod participant_id;
mod participation_kind;
mod participation_visibility;
mod role;
#[cfg(feature = "serde")]
mod signaling_module_frontend_data;
#[cfg(feature = "serde")]
mod signaling_module_peer_frontend_data;
mod target_participant;

pub use associated_participant::AssociatedParticipant;
pub use leave_reason::LeaveReason;
#[cfg(feature = "serde")]
pub use module_data::ModuleData;
#[cfg(feature = "serde")]
pub use module_peer_data::ModulePeerData;
pub use namespaced_command::NamespacedCommand;
pub use namespaced_event::NamespacedEvent;
pub use participant::Participant;
pub use participant_id::ParticipantId;
pub use participation_kind::ParticipationKind;
pub use participation_visibility::ParticipationVisibility;
pub use role::{ForRole, Role};
#[cfg(feature = "serde")]
pub use signaling_module_frontend_data::SignalingModuleFrontendData;
#[cfg(feature = "serde")]
pub use signaling_module_peer_frontend_data::SignalingModulePeerFrontendData;
pub use target_participant::TargetParticipant;
