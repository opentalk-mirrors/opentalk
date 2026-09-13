// @generated automatically by Diesel CLI.

diesel::table! {
    use crate::sql_types::*;

    assets (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 255]
        namespace -> Nullable<Varchar>,
        #[max_length = 255]
        kind -> Varchar,
        #[max_length = 512]
        filename -> Varchar,
        tenant_id -> Uuid,
        size -> Int8,
    }
}

diesel::table! {
    use crate::sql_types::*;

    event_dates (event_id) {
        event_id -> Uuid,
        starts_at -> Timestamptz,
        #[max_length = 255]
        starts_at_tz -> Varchar,
        ends_at -> Timestamptz,
        #[max_length = 255]
        ends_at_tz -> Varchar,
        is_all_day -> Bool,
        duration_secs -> Int4,
    }
}

diesel::table! {
    use crate::sql_types::*;

    event_email_invites (event_id, email) {
        event_id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        created_by -> Uuid,
        created_at -> Timestamptz,
        role -> EmailInviteRole,
    }
}

diesel::table! {
    use crate::sql_types::*;

    event_exceptions (id) {
        id -> Uuid,
        event_id -> Uuid,
        exception_date -> Timestamptz,
        #[max_length = 255]
        exception_date_tz -> Varchar,
        created_by -> Uuid,
        created_at -> Timestamptz,
        kind -> EventExceptionKind,
        #[max_length = 255]
        title -> Nullable<Varchar>,
        #[max_length = 4096]
        description -> Nullable<Varchar>,
        is_all_day -> Nullable<Bool>,
        starts_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        starts_at_tz -> Nullable<Varchar>,
        ends_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        ends_at_tz -> Nullable<Varchar>,
    }
}

diesel::table! {
    use crate::sql_types::*;

    event_favorites (user_id, event_id) {
        user_id -> Uuid,
        event_id -> Uuid,
    }
}

diesel::table! {
    use crate::sql_types::*;

    event_invites (id) {
        id -> Uuid,
        event_id -> Uuid,
        invitee -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamptz,
        status -> EventInviteStatus,
        role -> InviteRole,
    }
}

diesel::table! {
    use crate::sql_types::*;

    event_recurrences (event_id) {
        event_id -> Uuid,
        #[max_length = 4092]
        recurrence_pattern -> Varchar,
    }
}

diesel::table! {
    use crate::sql_types::*;

    event_shared_folders (event_id) {
        event_id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        path -> Text,
        write_url -> Text,
        write_password -> Text,
        read_url -> Text,
        read_password -> Text,
        provider_data -> Jsonb,
    }
}

diesel::table! {
    use crate::sql_types::*;

    event_training_participation_report_parameter_sets (event_id) {
        event_id -> Uuid,
        initial_checkpoint_delay_after -> Int8,
        initial_checkpoint_delay_within -> Int8,
        checkpoint_interval_after -> Int8,
        checkpoint_interval_within -> Int8,
    }
}

diesel::table! {
    use crate::sql_types::*;

    events (id) {
        id -> Uuid,
        id_serial -> Int8,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 4096]
        description -> Varchar,
        room -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_by -> Uuid,
        updated_at -> Timestamptz,
        is_adhoc -> Bool,
        tenant_id -> Uuid,
        revision -> Int4,
        show_meeting_details -> Bool,
    }
}

diesel::table! {
    use crate::sql_types::*;

    external_tariffs (external_id) {
        external_id -> Text,
        tariff_id -> Uuid,
    }
}

diesel::table! {
    use crate::sql_types::*;

    groups (id) {
        id -> Uuid,
        id_serial -> Int8,
        name -> Text,
        tenant_id -> Uuid,
    }
}

diesel::table! {
    use crate::sql_types::*;

    invites (id) {
        id -> Uuid,
        id_serial -> Int8,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_by -> Uuid,
        updated_at -> Timestamptz,
        room -> Uuid,
        active -> Bool,
        expiration -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use crate::sql_types::*;

    job_execution_logs (id) {
        id -> Int8,
        execution_id -> Int8,
        logged_at -> Timestamptz,
        log_level -> LogLevel,
        log_message -> Text,
    }
}

diesel::table! {
    use crate::sql_types::*;

    job_executions (id) {
        id -> Int8,
        job_id -> Int8,
        started_at -> Timestamptz,
        ended_at -> Nullable<Timestamptz>,
        job_status -> JobStatus,
    }
}

diesel::table! {
    use crate::sql_types::*;

    jobs (id) {
        id -> Int8,
        name -> Text,
        kind -> JobType,
        parameters -> Jsonb,
        timeout_secs -> Int4,
        recurrence -> Text,
    }
}

diesel::table! {
    use crate::sql_types::*;

    module_resources (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        room_id -> Uuid,
        created_by -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        #[max_length = 255]
        namespace -> Varchar,
        #[max_length = 255]
        tag -> Nullable<Varchar>,
        data -> Jsonb,
    }
}

diesel::table! {
    use crate::sql_types::*;

    refinery_schema_history (version) {
        version -> Int4,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        applied_on -> Nullable<Varchar>,
        #[max_length = 255]
        checksum -> Nullable<Varchar>,
    }
}

diesel::table! {
    use crate::sql_types::*;

    room_assets (room_id, asset_id) {
        room_id -> Uuid,
        asset_id -> Uuid,
    }
}

diesel::table! {
    use crate::sql_types::*;

    room_streaming_targets (id) {
        id -> Uuid,
        room_id -> Uuid,
        name -> Text,
        kind -> StreamingKind,
        streaming_endpoint -> Text,
        streaming_key -> Text,
        public_url -> Text,
    }
}

diesel::table! {
    use crate::sql_types::*;

    rooms (id) {
        id -> Uuid,
        id_serial -> Int8,
        created_by -> Uuid,
        created_at -> Timestamptz,
        #[max_length = 255]
        password -> Nullable<Varchar>,
        waiting_room -> Bool,
        tenant_id -> Uuid,
        e2e_encryption -> Bool,
        guest_access -> GuestAccess,
        #[max_length = 40]
        name -> Nullable<Varchar>,
        #[max_length = 64]
        suffix -> Nullable<Varchar>,
    }
}

diesel::table! {
    use crate::sql_types::*;

    sip_configs (id) {
        id -> Int8,
        room -> Uuid,
        #[max_length = 10]
        sip_id -> Varchar,
        #[max_length = 10]
        password -> Varchar,
        enable_lobby -> Bool,
    }
}

diesel::table! {
    use crate::sql_types::*;

    tariffs (id) {
        id -> Uuid,
        name -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        quotas -> Jsonb,
        disabled_modules -> Array<Nullable<Text>>,
        disabled_features -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    use crate::sql_types::*;

    tenants (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        oidc_tenant_id -> Text,
    }
}

diesel::table! {
    use crate::sql_types::*;

    user_groups (user_id, group_id) {
        user_id -> Uuid,
        group_id -> Uuid,
    }
}

diesel::table! {
    use crate::sql_types::*;

    users (id) {
        id -> Uuid,
        id_serial -> Int8,
        #[max_length = 255]
        oidc_sub -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        title -> Varchar,
        #[max_length = 255]
        firstname -> Varchar,
        #[max_length = 255]
        lastname -> Varchar,
        #[max_length = 35]
        language -> Nullable<Varchar>,
        #[max_length = 511]
        display_name -> Varchar,
        dashboard_theme -> Nullable<Theme>,
        conference_theme -> Nullable<Theme>,
        #[max_length = 30]
        phone -> Nullable<Varchar>,
        tenant_id -> Uuid,
        tariff_id -> Uuid,
        tariff_status -> TariffStatus,
        disabled_since -> Nullable<Timestamptz>,
        #[max_length = 2048]
        avatar_url -> Nullable<Varchar>,
        #[max_length = 255]
        timezone -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_authenticated_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(assets -> tenants (tenant_id));
diesel::joinable!(event_dates -> events (event_id));
diesel::joinable!(event_email_invites -> events (event_id));
diesel::joinable!(event_email_invites -> users (created_by));
diesel::joinable!(event_exceptions -> events (event_id));
diesel::joinable!(event_exceptions -> users (created_by));
diesel::joinable!(event_favorites -> events (event_id));
diesel::joinable!(event_favorites -> users (user_id));
diesel::joinable!(event_invites -> events (event_id));
diesel::joinable!(event_recurrences -> event_dates (event_id));
diesel::joinable!(event_shared_folders -> events (event_id));
diesel::joinable!(event_training_participation_report_parameter_sets -> events (event_id));
diesel::joinable!(events -> rooms (room));
diesel::joinable!(events -> tenants (tenant_id));
diesel::joinable!(external_tariffs -> tariffs (tariff_id));
diesel::joinable!(groups -> tenants (tenant_id));
diesel::joinable!(invites -> rooms (room));
diesel::joinable!(job_execution_logs -> job_executions (execution_id));
diesel::joinable!(job_executions -> jobs (job_id));
diesel::joinable!(module_resources -> rooms (room_id));
diesel::joinable!(module_resources -> tenants (tenant_id));
diesel::joinable!(module_resources -> users (created_by));
diesel::joinable!(room_assets -> assets (asset_id));
diesel::joinable!(room_assets -> rooms (room_id));
diesel::joinable!(room_streaming_targets -> rooms (room_id));
diesel::joinable!(rooms -> tenants (tenant_id));
diesel::joinable!(rooms -> users (created_by));
diesel::joinable!(sip_configs -> rooms (room));
diesel::joinable!(user_groups -> groups (group_id));
diesel::joinable!(user_groups -> users (user_id));
diesel::joinable!(users -> tariffs (tariff_id));
diesel::joinable!(users -> tenants (tenant_id));

diesel::allow_tables_to_appear_in_same_query!(
    assets,
    event_dates,
    event_email_invites,
    event_exceptions,
    event_favorites,
    event_invites,
    event_recurrences,
    event_shared_folders,
    event_training_participation_report_parameter_sets,
    events,
    external_tariffs,
    groups,
    invites,
    job_execution_logs,
    job_executions,
    jobs,
    module_resources,
    refinery_schema_history,
    room_assets,
    room_streaming_targets,
    rooms,
    sip_configs,
    tariffs,
    tenants,
    user_groups,
    users,
);
