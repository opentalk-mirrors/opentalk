// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::{
    AssetInventory, AuthorizationInventory, EventInventory, EventInviteInventory,
    EventSharedFolderInventory, EventTrainingParticipationReportInventory, GroupInventory,
    JobExecutionInventory, ModuleResourceInventory, RoomInventory, RoomInviteInventory,
    RoomSipConfigInventory, RoomStreamingTargetInventory, TariffInventory, TenantInventory,
    TransactionManager, UserInventory,
};

/// A connection to the inventory backend.
pub trait Inventory:
    Send
    + AssetInventory
    + AuthorizationInventory
    + EventInviteInventory
    + EventSharedFolderInventory
    + EventInventory
    + EventTrainingParticipationReportInventory
    + GroupInventory
    + JobExecutionInventory
    + ModuleResourceInventory
    + RoomInviteInventory
    + RoomSipConfigInventory
    + RoomInventory
    + RoomStreamingTargetInventory
    + TariffInventory
    + TenantInventory
    + UserInventory
    + TransactionManager
{
}

#[cfg(feature = "mockall")]
pub use mock::MockInventory;

#[cfg(feature = "mockall")]
mod mock {
    use std::{
        collections::{BTreeMap, BTreeSet},
        pin::Pin,
    };

    use futures_util::Stream;
    use opentalk_types_common::{
        assets::{AssetId, AssetSorting, FileSize},
        call_in::CallInId,
        events::{EventId, invites::EventInviteStatus},
        features::{FeatureId, ModuleFeatureId},
        module_resources::ModuleResourceId,
        modules::ModuleId,
        order::Ordering,
        pagination::{ItemCount, Page, PageSize},
        rooms::{RoomId, RoomIdOrAlias, invite_codes::InviteCode},
        streaming::{RoomStreamingTarget, StreamingTarget, StreamingTargetId},
        tariffs::TariffId,
        tenants::TenantId,
        time::Timestamp,
        training_participation_report::TrainingParticipationReportParameterSet,
        users::{GroupId, GroupName, UserId},
    };

    use crate::{
        Asset, AuthorizationUserRole, Event, EventEmailInvite, EventException, EventExceptionId,
        EventInvite, EventSharedFolder, EventTrainingParticipationReportParameterSet,
        ExternalTariffId, ExternalTariffMapping, GetEventExceptionsCursor, GetEventsCursor, Group,
        Job, JobExecution, JobExecutionId, JobId, ModuleResource, ModuleResourceFilter,
        ModuleResourceOperation, NewAsset, NewEvent, NewEventEmailInvite, NewEventException,
        NewEventInvite, NewEventSharedFolder, NewJobExecution, NewJobExecutionLog,
        NewModuleResource, NewRoom, NewRoomSipConfig, NewTariff, NewUser, OidcTenantId, Result,
        Room, RoomInvite, RoomSipConfig, RoomStreamingTargetRecord, Tariff, Tenant, UpdateAsset,
        UpdateEvent, UpdateEventEmailInvite, UpdateEventException, UpdateEventInvite,
        UpdateEventTrainingParticipationReportParameterSet, UpdateJobExecution, UpdateRoom,
        UpdateRoomSipConfig, UpdateRoomStreamingTarget, UpdateTariff, UpdateUser, UpsertOutcome,
        User,
    };

    /// An asset together with the room it belongs to, and the event that room
    /// is associated with, if any.
    type AssetWithRoomAndEvent = (Asset, RoomId, Option<EventId>);

    /// Contains events and associated items.
    type EventStream<'a> = Pin<
        Box<
            dyn Stream<
                    Item = Result<(
                        Event,
                        Option<EventInvite>,
                        Room,
                        Option<RoomSipConfig>,
                        bool,
                        Option<EventSharedFolder>,
                        Tariff,
                        Option<TrainingParticipationReportParameterSet>,
                    )>,
                > + 'a,
        >,
    >;

    /// Contains events and event exceptions.
    type EventAndExceptionStream<'a> =
        Pin<Box<dyn Stream<Item = Result<(EventException, Event)>> + 'a>>;

    /// An event and its related items.
    type EventWithRelatedItems = (
        Event,
        Option<EventInvite>,
        Room,
        Option<RoomSipConfig>,
        bool,
        Option<EventSharedFolder>,
        Tariff,
        Option<EventTrainingParticipationReportParameterSet>,
    );

    mockall::mock! {
        pub Inventory { }

        #[async_trait::async_trait]
        impl crate::AssetInventory for Inventory {
            async fn create_asset_for_room(&mut self, room_id: RoomId, asset: NewAsset) -> Result<Asset>;

            async fn delete_asset_from_room(
                &mut self,
                room_id: RoomId,
                asset_id: AssetId,
            ) -> Result<FileSize>;

            async fn get_asset_for_room(
                &mut self,
                room: &RoomIdOrAlias,
                asset_id: AssetId,
            ) -> Result<Asset>;

            async fn get_all_assets_with_size(&mut self) -> Result<Vec<(AssetId, i64)>>;

            async fn get_all_assets_for_room_paginated(
                &mut self,
                room: &RoomIdOrAlias,
                per_page: PageSize,
                page: Page,
            ) -> Result<(Vec<Asset>, ItemCount)>;

            async fn get_all_asset_ids_for_room(&mut self, room_id: RoomId) -> Result<Vec<AssetId>>;

            async fn get_all_assets_for_room_owner_paginated_ordered(
                &mut self,
                user_id: UserId,
                limit: PageSize,
                page: Page,
                sort: AssetSorting,
                order: Ordering,
            ) -> Result<(Vec<AssetWithRoomAndEvent>, ItemCount)>;

            async fn update_asset(
                &mut self,
                asset_id: AssetId,
                asset: UpdateAsset,
            ) -> Result<Option<Asset>>;

            async fn delete_asset_by_id_internal(&mut self, asset_id: AssetId) -> Result<()>;

            async fn delete_assets_by_ids(&mut self, asset_ids: &[AssetId]) -> Result<()>;
        }

        #[async_trait::async_trait]
        impl crate::AuthorizationInventory for Inventory {
            async fn get_event_user_role(
                &mut self,
                event_id: EventId,
                user_id: UserId,
                disabled_features: BTreeSet<ModuleFeatureId>,
                module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
            ) -> Result<AuthorizationUserRole>;

            async fn get_event_guest_allowed(
                &mut self,
                event_id: EventId,
                disabled_features: BTreeSet<ModuleFeatureId>,
                module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
            ) -> Result<bool>;

            async fn get_room_user_role(
                &mut self,
                room_id_or_alias: &RoomIdOrAlias,
                user_id: UserId,
                disabled_features: BTreeSet<ModuleFeatureId>,
                module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
            ) -> Result<AuthorizationUserRole>;

            async fn get_room_guest_allowed(
                &mut self,
                room_id_or_alias: &RoomIdOrAlias,
                disabled_features: BTreeSet<ModuleFeatureId>,
                module_features: BTreeMap<ModuleId, BTreeSet<FeatureId>>,
            ) -> Result<bool>;
        }

        #[async_trait::async_trait]
        impl crate::EventInviteInventory for Inventory {
            async fn try_create_event_email_invite(
                &mut self,
                invite: NewEventEmailInvite,
            ) -> Result<Option<EventEmailInvite>>;

            async fn try_create_event_invite(
                &mut self,
                invite: NewEventInvite,
            ) -> Result<Option<EventInvite>>;

            async fn get_event_invites_paginated(
                &mut self,
                event_id: EventId,
                per_page: PageSize,
                page: Page,
                filter_by_status: Option<EventInviteStatus>,
            ) -> Result<(Vec<(EventInvite, User)>, ItemCount)>;

            async fn get_event_email_invites_paginated(
                &mut self,
                event_id: EventId,
                per_page: PageSize,
                page: Page,
            ) -> Result<(Vec<EventEmailInvite>, ItemCount)>;

            async fn get_event_invite_for_user_and_room(
                &mut self,
                user_id: UserId,
                room_id: RoomId,
            ) -> Result<Option<EventInvite>>;

            async fn get_event_user_invites_for_events<'a>(
                &mut self,
                events: &[&'a Event],
            ) -> Result<Vec<Vec<(EventInvite, User)>>>;

            async fn get_event_email_invites_for_events<'a>(
                &mut self,
                events: &[&'a Event],
            ) -> Result<Vec<Vec<EventEmailInvite>>>;

            async fn get_invites_pending_for_user(&mut self, user_id: UserId) -> Result<Vec<EventInvite>>;

            async fn delete_event_invite_by_invitee(
                &mut self,
                event_id: EventId,
                user_id: UserId,
            ) -> Result<EventInvite>;

            async fn delete_event_invite_by_email(
                &mut self,
                event_id: EventId,
                email: &str,
            ) -> Result<EventEmailInvite>;

            async fn update_event_user_invite(
                &mut self,
                event_id: EventId,
                user_id: UserId,
                event_invite: UpdateEventInvite,
            ) -> Result<EventInvite>;

            async fn update_event_email_invite(
                &mut self,
                event_id: EventId,
                email: &str,
                event_invite: UpdateEventEmailInvite,
            ) -> Result<EventEmailInvite>;

            async fn migrate_event_email_invites_to_user_invites(
                &mut self,
                user: User,
            ) -> Result<Vec<(EventId, RoomId)>>;
        }

        #[async_trait::async_trait]
        impl crate::EventSharedFolderInventory for Inventory {
            async fn try_create_event_shared_folder(
                &mut self,
                new_shared_folder: NewEventSharedFolder,
            ) -> Result<Option<EventSharedFolder>>;

            async fn get_event_shared_folder(
                &mut self,
                event_id: EventId,
            ) -> Result<Option<EventSharedFolder>>;

            async fn get_event_shared_folders_for_room(
                &mut self,
                room_id: RoomId,
            ) -> Result<Vec<EventSharedFolder>>;

            async fn delete_shared_folder_by_event_id(&mut self, event_id: EventId) -> Result<()>;

            async fn delete_shared_folders_by_event_ids(&mut self, event_ids: &[EventId]) -> Result<()>;
        }

        #[async_trait::async_trait]
        impl crate::EventInventory for Inventory {
            async fn create_event(&mut self, new_event: NewEvent) -> Result<Event>;

            async fn update_event(&mut self, event_id: EventId, event: UpdateEvent) -> Result<Event>;

            async fn get_event(&mut self, event_id: EventId) -> Result<Event>;

            async fn get_event_for_room(
                &mut self,
                room_id_or_alias: &RoomIdOrAlias,
            ) -> Result<Option<Event>>;

            async fn get_event_id_for_room(&mut self, room_id: RoomId) -> Result<Option<EventId>>;

            async fn get_event_with_room_and_sip_config(
                &mut self,
                event_id: EventId,
            ) -> Result<(Event, Room, Option<RoomSipConfig>)>;

            async fn get_event_with_related_items(
                &mut self,
                user_id: UserId,
                event_id: EventId,
            ) -> Result<EventWithRelatedItems>;

            async fn get_all_events_updated_by_user(&mut self, user: UserId) -> Result<Vec<Event>>;

            async fn get_all_adhoc_event_and_room_ids_created_before(
                &mut self,
                created_before: Timestamp,
            ) -> Result<Vec<(EventId, RoomId)>>;

            async fn get_all_scheduled_event_and_room_ids_ended_before(
                &mut self,
                ended_before: Timestamp,
            ) -> Result<Vec<(EventId, RoomId)>>;

            async fn get_all_event_and_room_ids_created_by_user_including_disabled(
                &mut self,
                user_id: UserId,
            ) -> Result<Vec<(EventId, RoomId)>>;

            async fn get_all_finite_recurring_events(&mut self) -> Result<Vec<Event>>;

            async fn get_all_events_for_user(
                &mut self,
                user: User,
                only_recurring: bool,
            ) -> Result<Vec<Event>>;

            async fn get_all_events_for_user_paginated_as_stream<'a>(
                &'a mut self,
                user: User,
                only_favorites: bool,
                invite_status_filter: BTreeSet<EventInviteStatus>,
                time_min: Option<Timestamp>,
                time_max: Option<Timestamp>,
                created_before: Option<Timestamp>,
                created_after: Option<Timestamp>,
                adhoc: Option<bool>,
                time_independent: Option<bool>,
                cursor: Option<GetEventsCursor>,
            ) -> Result<EventStream<'a>>;

            async fn get_all_event_exceptions_for_user_paginated_as_stream<'a>(
                &'a mut self,
                user: User,
                only_favorites: bool,
                invite_status_filter: BTreeSet<EventInviteStatus>,
                time_min: Option<Timestamp>,
                time_max: Option<Timestamp>,
                created_before: Option<Timestamp>,
                created_after: Option<Timestamp>,
                adhoc: Option<bool>,
                time_independent: Option<bool>,
                cursor: Option<GetEventExceptionsCursor>,
            ) -> Result<EventAndExceptionStream<'a>>;

            async fn create_event_exception(
                &mut self,
                event_exception: NewEventException,
            ) -> Result<EventException>;

            async fn get_event_exceptions(
                &mut self,
                event_id: EventId,
                timestamps: &[Timestamp],
            ) -> Result<Vec<EventException>>;

            async fn get_event_exception(
                &mut self,
                event_id: EventId,
                instance_id_timestamp: Timestamp,
            ) -> Result<Option<EventException>>;

            async fn update_event_exception(
                &mut self,
                event_exception_id: EventExceptionId,
                event_exception: UpdateEventException,
            ) -> Result<EventException>;

            async fn delete_event_exceptions_for_event(&mut self, event_id: EventId) -> Result<()>;

            async fn delete_event_for_room(&mut self, room_id: RoomId) -> Result<()>;

            async fn create_event_favorite_for_user(
                &mut self,
                event_id: EventId,
                user_id: UserId,
            ) -> Result<bool>;

            async fn delete_event_favorite_for_user(
                &mut self,
                event_id: EventId,
                user_id: UserId,
            ) -> Result<bool>;
        }

        #[async_trait::async_trait]
        impl crate::EventTrainingParticipationReportInventory for Inventory {
            async fn get_event_training_participation_report_parameter_set(
                &mut self,
                event_id: EventId,
            ) -> Result<Option<EventTrainingParticipationReportParameterSet>>;

            async fn update_training_participation_report_parameter_set(
                &mut self,
                event_id: EventId,
                parameter_set: UpdateEventTrainingParticipationReportParameterSet,
            ) -> Result<EventTrainingParticipationReportParameterSet>;

            async fn try_create_event_training_participation_report_parameter_set(
                &mut self,
                parameter_set: EventTrainingParticipationReportParameterSet,
            ) -> Result<Option<EventTrainingParticipationReportParameterSet>>;

            async fn delete_event_training_participation_report_parameter_set(
                &mut self,
                event_id: EventId,
            ) -> Result<()>;
        }

        #[async_trait::async_trait]
        impl crate::GroupInventory for Inventory {
            async fn get_or_create_groups_by_name(
                &mut self,
                groups: &[(TenantId, GroupName)],
            ) -> Result<Vec<Group>>;
        }

        #[async_trait::async_trait]
        impl crate::JobExecutionInventory for Inventory {
            async fn get_job(&mut self, job_id: JobId) -> Result<Job>;

            async fn get_all_jobs(&mut self) -> Result<Vec<Job>>;

            async fn create_job_execution(
                &mut self,
                job_execution: NewJobExecution,
            ) -> Result<JobExecution>;

            async fn update_job_execution(
                &mut self,
                job_execution_id: JobExecutionId,
                job_execution: UpdateJobExecution,
            ) -> Result<JobExecution>;

            async fn create_job_execution_logs(
                &mut self,
                job_execution_logs: &[NewJobExecutionLog],
            ) -> Result<()>;
        }

        #[async_trait::async_trait]
        impl crate::ModuleResourceInventory for Inventory {
            async fn create_module_resource(
                &mut self,
                resource: NewModuleResource,
            ) -> Result<ModuleResource>;

            async fn get_module_resources(
                &mut self,
                resource_filter: ModuleResourceFilter,
            ) -> Result<Vec<ModuleResource>>;

            async fn patch_module_resources(
                &mut self,
                resource_filter: ModuleResourceFilter,
                operations: Vec<ModuleResourceOperation>,
            ) -> Result<Vec<ModuleResource>>;

            async fn get_all_module_ids_for_room(
                &mut self,
                room_id: RoomId,
            ) -> Result<Vec<ModuleResourceId>>;

            async fn delete_module_resources(
                &mut self,
                resource_filter: ModuleResourceFilter,
            ) -> Result<Vec<ModuleResource>>;

            async fn delete_all_module_resources_for_room(&mut self, room_id: RoomId) -> Result<()>;
        }

        #[async_trait::async_trait]
        impl crate::RoomInviteInventory for Inventory {
            async fn get_room_invite(&mut self, invite_code: InviteCode) -> Result<RoomInvite>;

            async fn get_room_invites_with_room_inactive_or_expired_before(
                &mut self,
                expired_before: Timestamp,
            ) -> Result<Vec<(InviteCode, RoomId)>>;
        }

        #[async_trait::async_trait]
        impl crate::RoomSipConfigInventory for Inventory {
            async fn get_room_sip_config(&mut self, room_id: RoomId) -> Result<Option<RoomSipConfig>>;

            async fn get_room_sip_config_with_room_and_creator(
                &mut self,
                call_in_id: CallInId,
            ) -> Result<Option<(RoomSipConfig, Room, User)>>;

            async fn create_room_sip_config(
                &mut self,
                sip_config: NewRoomSipConfig,
            ) -> Result<RoomSipConfig>;

            async fn update_room_sip_config(
                &mut self,
                room_id: RoomId,
                sip_config: UpdateRoomSipConfig,
            ) -> Result<Option<RoomSipConfig>>;

            async fn delete_room_sip_config(&mut self, room: &RoomIdOrAlias) -> Result<()>;
        }

        #[async_trait::async_trait]
        impl crate::RoomInventory for Inventory {
            async fn create_room(&mut self, new_room: NewRoom) -> Result<Room>;

            async fn get_room(&mut self, room: &RoomIdOrAlias) -> Result<Room>;

            async fn exists_room(&mut self, room: &RoomIdOrAlias) -> Result<bool>;

            async fn get_room_with_creator(&mut self, room: &RoomIdOrAlias) -> Result<(Room, User)>;

            async fn update_room(&mut self, room: &RoomIdOrAlias, update: UpdateRoom) -> Result<Room>;

            async fn delete_room(&mut self, room_id: RoomId) -> Result<()>;

            async fn get_all_orphaned_room_ids(&mut self) -> Result<Vec<RoomId>>;

            async fn get_rooms_accessible_to_user_with_creator_paginated(
                &mut self,
                user: UserId,
                limit: PageSize,
                page: Page,
            ) -> Result<(Vec<(Room, User)>, ItemCount)>;
        }

        #[async_trait::async_trait]
        impl crate::RoomStreamingTargetInventory for Inventory {
            async fn get_room_streaming_targets(
                &mut self,
                room_id: RoomId,
            ) -> Result<Vec<RoomStreamingTarget>>;

            async fn get_room_streaming_target_record(
                &mut self,
                room_id: RoomId,
                streaming_target_id: StreamingTargetId,
            ) -> Result<RoomStreamingTargetRecord>;

            async fn create_room_streaming_target(
                &mut self,
                room_id: RoomId,
                streaming_target: StreamingTarget,
            ) -> Result<RoomStreamingTarget>;

            async fn update_room_streaming_target(
                &mut self,
                room: &RoomIdOrAlias,
                streaming_target_id: StreamingTargetId,
                streaming_target: UpdateRoomStreamingTarget,
            ) -> Result<RoomStreamingTargetRecord>;

            async fn delete_room_streaming_target(
                &mut self,
                room: &RoomIdOrAlias,
                streaming_target_id: StreamingTargetId,
            ) -> Result<()>;

            async fn replace_room_streaming_targets(
                &mut self,
                room_id: RoomId,
                streaming_targets: Vec<StreamingTarget>,
            ) -> Result<Vec<RoomStreamingTarget>>;
        }

        #[async_trait::async_trait]
        impl crate::TariffInventory for Inventory {
            async fn get_all_tariffs(&mut self) -> Result<Vec<Tariff>>;

            async fn get_tariff(&mut self, tariff_id: TariffId) -> Result<Tariff>;

            async fn get_tariff_by_name(&mut self, name: &str) -> Result<Tariff>;

            async fn get_tariff_for_user(&mut self, user_id: UserId) -> Result<Tariff>;

            async fn get_tariff_by_external_tariff_id(
                &mut self,
                external_tariff_id: ExternalTariffId,
            ) -> Result<Option<Tariff>>;

            async fn get_all_external_tariff_ids_for_tariff(
                &mut self,
                tariff_id: TariffId,
            ) -> Result<Vec<ExternalTariffId>>;

            async fn create_tariff(&mut self, tariff: NewTariff) -> Result<Tariff>;

            async fn update_tariff(&mut self, tariff: Tariff, changeset: UpdateTariff) -> Result<Tariff>;

            async fn delete_tariff(&mut self, tariff_id: TariffId) -> Result<()>;

            async fn delete_all_external_tariff_mappings_for_tariff(
                &mut self,
                tariff_id: TariffId,
            ) -> Result<()>;

            async fn delete_external_tariff_mappings_for_tariff_by_external_id(
                &mut self,
                tariff_id: TariffId,
                external_tariff_ids: &[ExternalTariffId],
            ) -> Result<()>;

            async fn create_external_tariff_mapping(
                &mut self,
                external_tariff_id: ExternalTariffId,
                tariff_id: TariffId,
            ) -> Result<ExternalTariffMapping>;
        }

        #[async_trait::async_trait]
        impl crate::TenantInventory for Inventory {
            async fn get_tenant(&mut self, tenant_id: TenantId) -> Result<Tenant>;

            async fn get_or_create_tenant_by_oidc_id(
                &mut self,
                oidc_tenant_id: &OidcTenantId,
            ) -> Result<Tenant>;
        }

        #[async_trait::async_trait]
        impl crate::UserInventory for Inventory {
            async fn create_user(&mut self, new_user: NewUser) -> Result<User>;

            async fn get_user(&mut self, user_id: UserId) -> Result<User>;

            async fn update_user<'a>(&mut self, user_id: UserId, user: UpdateUser<'a>) -> Result<User>;

            async fn delete_user(&mut self, user_id: UserId) -> Result<()>;

            async fn set_last_authenticated_at_to_now(&mut self, user_id: UserId) -> Result<()>;

            async fn get_all_users(&mut self) -> Result<Vec<User>>;

            async fn get_users_by_ids(&mut self, user_ids: &[UserId]) -> Result<Vec<User>>;

            async fn get_user_ids_disabled_before(&mut self, timestamp: Timestamp) -> Result<Vec<UserId>>;

            async fn get_user_by_email(
                &mut self,
                tenant_id: TenantId,
                email_address: &str,
            ) -> Result<Option<User>>;

            async fn create_or_update_user_by_oidc_sub(
                &mut self,
                user: NewUser,
                enforce_display_name_on_update: bool,
            ) -> Result<UpsertOutcome<User>>;

            async fn get_users_by_oidc_subs<'a>(
                &mut self,
                tenant_id: TenantId,
                subs: &[&'a str],
            ) -> Result<Vec<User>>;

            async fn get_user_for_tenant(&mut self, tenant_id: TenantId, user_id: UserId) -> Result<User>;

            async fn add_user_to_groups(
                &mut self,
                user_id: UserId,
                groups: &[GroupId],
            ) -> Result<BTreeSet<GroupId>>;

            async fn remove_user_from_all_groups_except(
                &mut self,
                user_id: UserId,
                group_ids_to_keep: &[GroupId],
            ) -> Result<BTreeSet<GroupId>>;

            async fn remove_user_from_all_groups(&mut self, user_id: UserId) -> Result<()>;

            async fn get_user_storage_used_size_u64(&mut self, user_id: UserId) -> Result<u64>;

            async fn find_users(
                &mut self,
                tenant_id: TenantId,
                search_string: &str,
                limit: usize,
            ) -> Result<Vec<User>>;
        }

        #[async_trait::async_trait]
        impl crate::TransactionManager for Inventory {
            async fn begin_transaction(&mut self) -> Result<()>;

            async fn rollback_transaction(&mut self) -> Result<()>;

            async fn commit_transaction(&mut self) -> Result<()>;
        }
    }

    impl crate::Inventory for MockInventory {}
}
