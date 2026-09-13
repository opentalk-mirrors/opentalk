// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Facade library for accessing the OpenTalk data inventory.
//!
//! The production inventory implementation will usually be backed by a database
//! This abstraction helps to maintain proper separation of concerns, and to
//! keep the usage of it testable using e.g. mocks.

mod asset;
mod authorization;
mod event;
mod event_invite;
mod event_shared_folder;
mod event_training_participation_report;
mod group;
mod has_users;
mod inventory;
mod inventory_provider;
mod job_execution;
mod module_resource;
mod room;
mod room_invite;
mod room_sip_config;
mod room_streaming_target;
mod tariff;
mod tenant;
mod transaction;
mod transaction_manager;
mod upsert;
mod user;

pub mod utils;

pub use asset::{Asset, AssetInventory, NewAsset, UpdateAsset};
pub use authorization::{AuthorizationInventory, AuthorizationUserRole};
pub use event::{
    Event, EventDate, EventException, EventExceptionId, EventExceptionKind, EventInventory,
    EventRecurrence, GetEventExceptionsCursor, GetEventsCursor, NewEvent, NewEventDate,
    NewEventException, NewEventRecurrence, UpdateEvent, UpdateEventDate, UpdateEventException,
    UpdateEventRecurrence,
};
pub use event_invite::{
    EventEmailInvite, EventInvite, EventInviteId, EventInviteInventory, NewEventEmailInvite,
    NewEventInvite, UpdateEventEmailInvite, UpdateEventInvite,
};
pub use event_shared_folder::{
    EventSharedFolder, EventSharedFolderInventory, NewEventSharedFolder, OpencloudShareReference,
    SharedFolderProvider,
};
pub use event_training_participation_report::{
    EventTrainingParticipationReportInventory, EventTrainingParticipationReportParameterSet,
    UpdateEventTrainingParticipationReportParameterSet,
};
pub use group::{Group, GroupInventory};
pub use has_users::HasUsers;
pub use inventory::Inventory;
pub use inventory_provider::InventoryProvider;
pub use job_execution::{
    Job, JobExecution, JobExecutionId, JobExecutionInventory, JobExecutionLogLevel, JobId,
    JobStatus, JobType, NewJobExecution, NewJobExecutionLog, UpdateJobExecution,
};
pub use module_resource::{
    ModuleResource, ModuleResourceFilter, ModuleResourceInventory, ModuleResourceOperation,
    NewModuleResource,
};
/// The result type typically used for functions in this crate.
pub use opentalk_inventory_common::{
    Result,
    error::{Error, InventoryBackendError},
};
pub use room::{NewRoom, Room, RoomInventory, UpdateRoom};
pub use room_invite::{RoomInvite, RoomInviteInventory};
pub use room_sip_config::{
    NewRoomSipConfig, RoomSipConfig, RoomSipConfigInventory, UpdateRoomSipConfig,
};
pub use room_streaming_target::{
    RoomStreamingTargetInventory, RoomStreamingTargetRecord, UpdateRoomStreamingTarget,
};
pub use tariff::{
    ExternalTariffId, ExternalTariffMapping, NewTariff, Tariff, TariffInventory, UpdateTariff,
};
pub use tenant::{OidcTenantId, Tenant, TenantInventory};
pub use transaction::transaction;
pub use transaction_manager::TransactionManager;
pub use upsert::UpsertOutcome;
pub use user::{NewUser, UpdateUser, User, UserInventory};
#[cfg(feature = "mockall")]
pub use {authorization::MockAuthorizationInventory, inventory_provider::MockInventoryProvider};
