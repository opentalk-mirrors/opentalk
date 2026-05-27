<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Configuration Reference (Auto-generated)

Last updated: 2026-04-22

This document lists the configuration variables for each Ansible role.
Variables are collected from `defaults/*.yml` and `vars/*.yml`.
Default values should be overridden in `/etc/opentalk-compose/settings.yml`.

## Global Variables in settings.yml

| Variable | Type | Default Value |
|----------|------|---------------|
| `compose_project_name` | string | `opentalk` |
| `compose_service_etherpad_enabled` | boolean | `true` |
| `compose_service_keycloak_enabled` | boolean | `true` |
| `compose_service_livekit_server_enabled` | boolean | `true` |
| `compose_service_mailcrab_enabled` | boolean | `false` |
| `compose_service_opentalk_controller_enabled` | boolean | `true` |
| `compose_service_opentalk_obelisk_enabled` | boolean | `false` |
| `compose_service_opentalk_outlook_add_in_enabled` | boolean | `false` |
| `compose_service_opentalk_recorder_enabled` | boolean | `true` |
| `compose_service_opentalk_smtp_mailer_enabled` | boolean | `false` |
| `compose_service_opentalk_web_frontend_enabled` | boolean | `true` |
| `compose_service_rabbitmq_enabled` | boolean | `true` |
| `compose_service_spacedeck_enabled` | boolean | `true` |
| `compose_service_traefik_enabled` | boolean | `true` |
| `config_dir` | string | `/etc/opentalk-compose` |
| `deploy_dir` | string | `/opt/opentalk-compose` |
| `ot_domain` | string | `` |
| `ot_ingress_port` | integer | `443` |
| `ot_ingress_protocol` | string | `https` |
| `s3_storage_provider` | string | `garage` |
| `services_dir` | string | `/opt/opentalk-compose/services` |

## opentalk_compose.backup

| Variable | Type | Default Value |
|----------|------|---------------|
| `backup_dir` | string | `/var/opt/opentalk-compose/backups` |
| `backup_dir_config` | string | `{{ backup_dir }}/config` |
| `backup_dir_sql_dumps` | string | `{{ backup_dir }}/dumps` |
| `backup_dir_volumes` | string | `{{ backup_dir }}/volumes` |
| `backup_docker_volume_list` | string | `{{`<br>`  ( [compose_project_name ~ '_garage_meta', compose_project_name ~ '_garage_data']`<br>`    if (s3_storage_provider \| default('garage')) == 'garage'`<br>`    else [compose_project_name ~ '_minio'] )`<br>`  + [compose_project_name ~ '_rabbitmq', compose_project_name ~ '_traefik']`<br>`}}` |
| `backup_docker_volume_restore_files` | list | `[]` |
| `backup_retention_max_age_days` | integer | `7` |
| `backup_retention_max_items` | integer | `5` |
| `config_backup_file_name` | string | `opentalk-compose_config_backup-{{ lookup('pipe','date +%Y%m%d-%H%M%S') }}.tar.gz` |
| `config_backup_restore_file` | string | `` |
| `controller_db_dump_file` | string | `{{ backup_dir_sql_dumps }}/controller-{{ lookup('pipe','date +%Y%m%d-%H%M%S') }}.sql` |
| `controller_db_dump_restore_file` | string | `` |
| `controller_db_host` | string | `{{ controller_database_host \| default('controller-db') }}` |
| `controller_db_name` | string | `{{ controller_database_name \| default('controller') }}` |
| `controller_db_password` | string | `{{ CONTROLLER_DB_PASSWORD }}` |
| `controller_db_port` | string | `{{ controller_database_port \| default(5432) }}` |
| `controller_db_scheduled_backup_enabled` | boolean | `true` |
| `controller_db_scheduled_backup_time` | string | `02:00` |
| `controller_db_user` | string | `{{ controller_database_user \| default('ot') }}` |
| `keycloak_db_dump_file` | string | `{{ backup_dir_sql_dumps }}/keycloak-{{ lookup('pipe','date +%Y%m%d-%H%M%S') }}.sql` |
| `keycloak_db_host` | string | `{{ keycloak_database_host \| default('keycloak-db') }}` |
| `keycloak_db_name` | string | `{{ keycloak_database_name \| default('keycloak') }}` |
| `keycloak_db_password` | string | `{{ KEYCLOAK_DB_PASSWORD }}` |
| `keycloak_db_port` | string | `{{ keycloak_database_port \| default(5432) }}` |
| `keycloak_db_user` | string | `{{ keycloak_database_user \| default('ot') }}` |

## opentalk_compose.docker_env

| Variable | Type | Default Value |
|----------|------|---------------|
| `docker_network_driver` | string | `bridge` |
| `docker_network_name` | string | `opentalk_internal` |
| `docker_network_state` | string | `present` |
| `docker_overwrite_main_compose_file` | boolean | `false` |

## opentalk_compose.etherpad

| Variable | Type | Default Value |
|----------|------|---------------|
| `etherpad_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `etherpad_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `etherpad_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `etherpad_docker_compose_services_state` | string | `{{ compose_service_etherpad_state \| default('present') }}` |
| `etherpad_docker_compose_ulimits_nofile_hard` | integer | `16384` |
| `etherpad_docker_compose_ulimits_nofile_soft` | integer | `8192` |
| `etherpad_docker_compose_ulimits_nproc` | integer | `4096` |
| `etherpad_docker_environment_default_pad_text` | string | `Welcome to OpenTalk` |
| `etherpad_docker_environment_suppress_errors_in_pad_text` | boolean | `true` |
| `etherpad_docker_environment_trust_proxy` | boolean | `true` |
| `etherpad_docker_extra_hosts` | list | `[]` |
| `etherpad_docker_image_src` | string | `registry.heinlein.group/opentalk/etherpad` |
| `etherpad_docker_labels_traefik_enable` | boolean | `true` |
| `etherpad_docker_labels_traefik_http_routers_etherpad_rule` | string | `Host(`{{ ot_domain }}`) && PathPrefix(`/etherpad`)` |
| `etherpad_docker_ports` | list | `[]` |

## opentalk_compose.garage

| Variable | Type | Default Value |
|----------|------|---------------|
| `garage_admin_api_bind_addr` | string | `[::]:3903` |
| `garage_admin_api_port` | integer | `3903` |
| `garage_admin_metrics_require_token` | string | `false` |
| `garage_admin_metrics_token` | null | `null` |
| `garage_admin_token_file` | string | `/run/secrets/garage_admin_token` |
| `garage_admin_trace_sink` | string | `http://localhost:4317` |
| `garage_allow_punycode` | string | `false` |
| `garage_allow_world_readable_secrets` | string | `false` |
| `garage_bind_host` | string | `127.0.0.1` |
| `garage_block_max_concurrent_reads` | string | `16` |
| `garage_block_max_concurrent_writes_per_request` | string | `10` |
| `garage_block_ram_buffer_max` | string | `256MiB` |
| `garage_block_size` | string | `1M` |
| `garage_bootstrap_peers` | list | `[peer_id@host:port]` |
| `garage_bucket_name` | string | `opentalk` |
| `garage_compression_level` | string | `1` |
| `garage_consistency_mode` | string | `consistent` |
| `garage_consul_api` | string | `catalog` |
| `garage_consul_ca_cert` | string | `/etc/consul/consul-ca.crt` |
| `garage_consul_client_cert` | string | `/etc/consul/consul-client.crt` |
| `garage_consul_client_key` | string | `/etc/consul/consul-client.key` |
| `garage_consul_datacenters` | string | `[]` |
| `garage_consul_http_addr` | string | `https://127.0.0.1:8500` |
| `garage_consul_service_name` | string | `garage-daemon` |
| `garage_consul_tags` | string | `[]` |
| `garage_consul_tls_skip_verify` | string | `false` |
| `garage_consul_token` | null | `null` |
| `garage_data_dir` | string | `/var/lib/garage/data` |
| `garage_data_fsync` | string | `false` |
| `garage_db_engine` | string | `lmdb` |
| `garage_disable_scrub` | string | `false` |
| `garage_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `garage_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `garage_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `garage_docker_compose_services_state` | string | `{{ compose_service_garage_state \| default('present') }}` |
| `garage_docker_image_src` | string | `registry.heinlein.group/opentalk/garage` |
| `garage_k8s_namespace` | string | `garage` |
| `garage_k8s_service_name` | string | `garage-daemon` |
| `garage_k8s_skip_crd` | string | `false` |
| `garage_key_name` | string | `opentalk-key` |
| `garage_layout_capacity` | string | `32G` |
| `garage_layout_zone` | string | `dc1` |
| `garage_lmdb_map_size` | string | `1T` |
| `garage_metadata_auto_snapshot_interval` | string | `6h` |
| `garage_metadata_dir` | string | `/var/lib/garage/meta` |
| `garage_metadata_fsync` | string | `true` |
| `garage_metadata_snapshots_dir` | string | `/var/lib/garage/snapshots` |
| `garage_replication_factor` | integer | `1` |
| `garage_rpc_bind_addr` | string | `[::]:3901` |
| `garage_rpc_bind_outgoing` | string | `false` |
| `garage_rpc_port` | integer | `3901` |
| `garage_rpc_public_addr` | string | `garage:3901` |
| `garage_rpc_public_addr_subnet` | string | `192.168.0.0/24` |
| `garage_rpc_secret_file` | string | `/run/secrets/garage_rpc_secret` |
| `garage_s3_api_bind_addr` | string | `[::]:3900` |
| `garage_s3_api_port` | integer | `3900` |
| `garage_s3_api_root_domain` | string | `.s3.garage.localhost` |
| `garage_s3_region` | string | `unknown` |
| `garage_s3_web_add_host_to_metrics` | string | `false` |
| `garage_s3_web_bind_addr` | string | `[::]:3902` |
| `garage_s3_web_index` | string | `index.html` |
| `garage_s3_web_port` | string | `3902` |
| `garage_s3_web_root_domain` | string | `.web.garage` |
| `garage_use_local_tz` | string | `false` |

## opentalk_compose.import_users

| Variable | Type | Default Value |
|----------|------|---------------|
| `import_users_default_password` | null | `null` |
| `import_users_enabled` | boolean | `true` |
| `import_users_file` | string | `/path/to/users.csv` |
| `import_users_force_password_reset_email` | boolean | `false` |
| `import_users_password_length` | integer | `20` |
| `import_users_password_temporary` | boolean | `true` |
| `import_users_realm` | string | `{{ keycloak_realm \| default('opentalk') }}` |
| `keycloak_auth_password` | string | `{{ keycloak_permanent_admin_password \| default(KC_PERMANENT_ADMIN_PASSWORD) }}` |
| `keycloak_auth_realm` | string | `master` |
| `keycloak_auth_username` | string | `{{ keycloak_permanent_admin_username \| default('admin') }}` |
| `keycloak_base_url` | string | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) \| build_subdomain_url }}/auth` |
| `keycloak_local_hostname` | string | `` |
| `keycloak_local_port` | string | `{{ ot_ingress_port }}` |

## opentalk_compose.keycloak

| Variable | Type | Default Value |
|----------|------|---------------|
| `accessCodeLifespan` | string | `Max time a client has to finish auth code flow (default: 1 minute)` |
| `accessCodeLifespanLogin` | string | `Max time user has to complete login (default: 30 minutes)` |
| `accessCodeLifespanUserAction` | string | `Max time for user action during login (default: 5 minutes)` |
| `accessTokenLifespan` | string | `How long an access token is valid (default: 5 minutes)` |
| `accessTokenLifespanForImplicitFlow` | string | `Access token lifespan for implicit flow (default: 15 minutes)` |
| `actionTokenGeneratedByAdminLifespan` | string | `Default lifespan for admin-initiated action tokens (default: 12 hours)` |
| `actionTokenGeneratedByUserLifespan` | string | `Lifespan for user-initiated action tokens (default: 5 minutes)` |
| `keycloak_account_theme` | string | `opentalk` |
| `keycloak_api_base_url` | string | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) \| build_subdomain_url }}/auth/admin/realms/{{ keycloak_realm }}` |
| `keycloak_auth_password` | string | `{{ keycloak_permanent_admin_password }}` |
| `keycloak_auth_realm` | string | `master` |
| `keycloak_auth_username` | string | `{{ keycloak_permanent_admin_username }}` |
| `keycloak_backchannel_logout_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/controller/v1/auth/logout` |
| `keycloak_base_url` | string | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) \| build_subdomain_url }}/auth` |
| `keycloak_bootstrap_admin_password` | string | `{{ KC_BOOTSTRAP_ADMIN_PASSWORD }}` |
| `keycloak_bootstrap_admin_remove` | boolean | `true` |
| `keycloak_bootstrap_admin_username` | string | `bootstrap_admin` |
| `keycloak_db_bind_interface` | string | `127.0.0.1` |
| `keycloak_db_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `keycloak_db_docker_compose_ulimits_nofile_soft` | integer | `65536` |
| `keycloak_db_docker_compose_ulimits_nproc` | integer | `8192` |
| `keycloak_db_host_port` | integer | `5433` |
| `keycloak_db_password` | string | `{{ KEYCLOAK_DB_PASSWORD }}` |
| `keycloak_db_schema` | string | `public` |
| `keycloak_db_url_database` | string | `keycloak` |
| `keycloak_db_url_host` | string | `keycloak-db` |
| `keycloak_db_url_port` | integer | `5432` |
| `keycloak_db_username` | string | `ot` |
| `keycloak_db_vendor` | string | `postgres` |
| `keycloak_demo_user_email` | string | `demo.user@example.com` |
| `keycloak_demo_user_enabled` | boolean | `true` |
| `keycloak_demo_user_firstName` | string | `Test` |
| `keycloak_demo_user_force` | string | `false` |
| `keycloak_demo_user_lastName` | string | `User` |
| `keycloak_demo_user_name` | string | `demo` |
| `keycloak_demo_user_password` | string | `{{ DEMO_USER_PASSWORD }}` |
| `keycloak_demo_user_state` | string | `present` |
| `keycloak_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `keycloak_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `keycloak_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `keycloak_docker_compose_services_state` | string | `{{ compose_service_keycloak_state \| default('present') }}` |
| `keycloak_docker_compose_state` | string | `present` |
| `keycloak_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `keycloak_docker_compose_ulimits_nofile_soft` | integer | `16384` |
| `keycloak_docker_compose_ulimits_nproc` | integer | `8192` |
| `keycloak_docker_image_src` | string | `registry.heinlein.group/opentalk/keycloak` |
| `keycloak_health_enabled` | string | `true` |
| `keycloak_health_url` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_auth_realm }}/.well-known/openid-configuration` |
| `keycloak_hostname` | string | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) \| build_subdomain_url }}/auth` |
| `keycloak_http_relative_path` | string | `/auth` |
| `keycloak_local_hostname` | string | `` |
| `keycloak_local_port` | string | `{{ ot_ingress_port }}` |
| `keycloak_login_theme` | string | `opentalk` |
| `keycloak_manage_enabled` | boolean | `true` |
| `keycloak_metrics_enabled` | string | `true` |
| `keycloak_permanent_admin_email` | string | `admin@{{ ot_domain \| default('localhost') }}` |
| `keycloak_permanent_admin_firstname` | string | `Keycloak` |
| `keycloak_permanent_admin_lastname` | string | `Administrator` |
| `keycloak_permanent_admin_password` | string | `{{ KC_PERMANENT_ADMIN_PASSWORD }}` |
| `keycloak_permanent_admin_username` | string | `admin` |
| `keycloak_proxy` | string | `edge` |
| `keycloak_proxy_headers` | string | `xforwarded` |
| `keycloak_realm` | string | `opentalk` |
| `keycloak_realm_access_code_lifespan` | integer | `60` |
| `keycloak_realm_access_code_lifespan_login` | integer | `1800` |
| `keycloak_realm_access_code_lifespan_user_action` | integer | `300` |
| `keycloak_realm_access_token_lifespan` | integer | `300` |
| `keycloak_realm_access_token_lifespan_implicit` | integer | `900` |
| `keycloak_realm_action_token_generated_by_admin_lifespan` | integer | `43200` |
| `keycloak_realm_action_token_generated_by_user_lifespan` | integer | `300` |
| `keycloak_realm_client_session_idle_timeout` | integer | `86400` |
| `keycloak_realm_client_session_max_lifespan` | integer | `2592000` |
| `keycloak_realm_offline_session_idle_timeout` | integer | `2592000` |
| `keycloak_realm_offline_session_max_lifespan` | integer | `5184000` |
| `keycloak_realm_offline_session_max_lifespan_enabled` | boolean | `true` |
| `keycloak_realm_refresh_token_max_reuse` | integer | `0` |
| `keycloak_realm_revoke_refresh_token` | boolean | `false` |
| `keycloak_realm_sso_session_idle_timeout` | integer | `86400` |
| `keycloak_realm_sso_session_max_lifespan` | integer | `2592000` |
| `keycloak_redirectUris_uris` | list | `[{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/auth/popup_callback, {{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/auth/callback, {{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/dashboard, {{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/dashboard/*, {{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/room/*, {{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/join, {{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/]` |
| `keycloak_spi_sticky_session_encoder_infinispan_should_attach_route` | string | `false` |
| `keycloak_theme_path` | string | `{{ services_dir }}/keycloak/keycloak-ot-theme.jar` |
| `keycloak_wait_to_be_reachable_delay` | integer | `10` |
| `keycloak_wait_to_be_reachable_retries` | integer | `20` |
| `offlineSessionIdleTimeout` | string | `Idle timeout for offline tokens (default: 30 days)` |
| `offlineSessionMaxLifespan` | string | `Max lifetime for offline tokens (default: 60 days)` |
| `postgres_docker_image_src` | string | `registry.heinlein.group/opentalk/postgres` |
| `refreshTokenMaxReuse` | string | `How many times a refresh token can be reused (0 = unlimited, only if revokeRefreshToken is false)` |
| `revokeRefreshToken` | string | `If true, refresh tokens can only be used once (default: false)` |
| `ssoSessionIdleTimeout` | string | `Time before idle session expires (default: 24 hours)` |
| `ssoSessionMaxLifespan` | string | `Maximum session lifetime (default: 30 days)` |

## opentalk_compose.livekit_server

| Variable | Type | Default Value |
|----------|------|---------------|
| `livekit_api_key_controller` | string | `{{ LIVEKIT_API_SECRET }}` |
| `livekit_audio_active_level` | string | `30` |
| `livekit_audio_active_red_encoding` | string | `true` |
| `livekit_audio_min_percentile` | string | `40` |
| `livekit_audio_smooth_intervals` | string | `4` |
| `livekit_audio_update_interval` | string | `500` |
| `livekit_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `livekit_docker_compose_ulimits_nofile_hard` | integer | `98304` |
| `livekit_docker_compose_ulimits_nofile_soft` | integer | `98304` |
| `livekit_docker_compose_ulimits_nproc` | integer | `16384` |
| `livekit_docker_image_src` | string | `registry.heinlein.group/opentalk/livekit-server` |
| `livekit_ingress_rtmp_base_url` | string | `rtmp://my.domain.com/live` |
| `livekit_ingress_whip_base_url` | string | `http://my.domain.com/whip` |
| `livekit_limit_bytes_per_sec` | string | `1000000000` |
| `livekit_limit_max_attributes_size` | string | `0` |
| `livekit_limit_max_metadata_size` | string | `0` |
| `livekit_limit_max_participant_identity_length` | string | `0` |
| `livekit_limit_max_room_name_length` | string | `0` |
| `livekit_limit_num_tracks` | string | `-1` |
| `livekit_limit_subscription_limit_audio` | string | `0` |
| `livekit_limit_subscription_limit_video` | string | `0` |
| `livekit_logging_json` | string | `false` |
| `livekit_logging_level` | string | `info` |
| `livekit_logging_pion_level` | string | `error` |
| `livekit_logging_sample` | string | `false` |
| `livekit_node_selector_algorithm` | string | `lowest` |
| `livekit_node_selector_kind` | string | `sysload` |
| `livekit_node_selector_regions` | list | `[{name: us-west-2, lat: 44.19, lon: -123.07}]` |
| `livekit_node_selector_sort_by` | string | `sysload` |
| `livekit_node_selector_sysload_limit` | string | `0.7` |
| `livekit_port` | integer | `7880` |
| `livekit_prometheus_port` | string | `6789` |
| `livekit_psrpc_backoff` | string | `500ms` |
| `livekit_psrpc_buffer_size` | string | `1000` |
| `livekit_psrpc_max_attempts` | string | `3` |
| `livekit_psrpc_timeout` | string | `500ms` |
| `livekit_redis_address` | string | `redis.host:6379` |
| `livekit_redis_cluster_addresses` | list | `[livekit-redis-node-0:6379]` |
| `livekit_redis_db` | string | `0` |
| `livekit_redis_password` | null | `null` |
| `livekit_redis_sentinel_addresses` | list | `[livekit-redis-node-0:26379]` |
| `livekit_redis_sentinel_master_name` | string | `livekit` |
| `livekit_redis_sentinel_password` | null | `null` |
| `livekit_redis_sentinel_username` | null | `null` |
| `livekit_redis_tls_ca_cert_file` | null | `null` |
| `livekit_redis_tls_client_cert_file` | null | `null` |
| `livekit_redis_tls_client_key_file` | null | `null` |
| `livekit_redis_tls_enabled` | string | `true` |
| `livekit_redis_tls_insecure` | string | `false` |
| `livekit_redis_tls_server_name` | null | `null` |
| `livekit_redis_username` | null | `null` |
| `livekit_region` | string | `us-west-2` |
| `livekit_room_auto_create` | string | `true` |
| `livekit_room_departure_timeout` | string | `20` |
| `livekit_room_empty_timeout` | string | `300` |
| `livekit_room_enable_remote_unmute` | string | `false` |
| `livekit_room_enabled_codecs` | list | `[audio/opus, video/vp8]` |
| `livekit_room_max_participants` | string | `0` |
| `livekit_room_playout_delay_enabled` | string | `false` |
| `livekit_room_playout_delay_max` | string | `2000` |
| `livekit_room_playout_delay_min` | string | `100` |
| `livekit_room_sync_streams` | string | `false` |
| `livekit_rtc_allow_tcp_fallback` | string | `true` |
| `livekit_rtc_batch_io_batch_size` | string | `128` |
| `livekit_rtc_batch_io_max_flush_interval` | string | `2ms` |
| `livekit_rtc_congestion_control_allow_pause` | string | `true` |
| `livekit_rtc_congestion_control_enabled` | string | `true` |
| `livekit_rtc_data_channel_max_buffered_amount` | string | `0` |
| `livekit_rtc_enable_loopback_candidate` | boolean | `false` |
| `livekit_rtc_interfaces_excludes` | list | `[docker0]` |
| `livekit_rtc_interfaces_includes` | list | `[en0]` |
| `livekit_rtc_ips_excludes` | list | `[10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16]` |
| `livekit_rtc_ips_includes` | list | `[]` |
| `livekit_rtc_node_ip` | string | `{{ ansible_host }}` |
| `livekit_rtc_packet_buffer_size_audio` | string | `200` |
| `livekit_rtc_packet_buffer_size_video` | string | `500` |
| `livekit_rtc_pli_throttle_high_quality` | string | `1s` |
| `livekit_rtc_pli_throttle_low_quality` | string | `500ms` |
| `livekit_rtc_pli_throttle_mid_quality` | string | `1s` |
| `livekit_rtc_port_range_end` | integer | `65535` |
| `livekit_rtc_port_range_start` | integer | `50000` |
| `livekit_rtc_strict_acks` | string | `true` |
| `livekit_rtc_stun_servers` | list | `[stun.l.google.com:5349]` |
| `livekit_rtc_tcp_port` | integer | `7881` |
| `livekit_rtc_turn_servers` | list | `[{host: turn.example.com, port: 443, protocol: tls, username: , credential: }, {host: turn.example.com, port: 443, protocol: tcp, username: , credential: }, {host: turn.example.com, port: 443, protocol: udp, username: , credential: }]` |
| `livekit_rtc_udp_port` | string | `7882-7890` |
| `livekit_rtc_use_external_ip` | boolean | `true` |
| `livekit_rtc_use_ice_lite` | string | `false` |
| `livekit_rtc_use_mdns` | string | `false` |
| `livekit_server_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `livekit_server_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `livekit_server_docker_compose_services_state` | string | `{{ compose_service_livekit_server_state \| default('present') }}` |
| `livekit_signal_relay_max_retry_interval` | string | `5s` |
| `livekit_signal_relay_min_retry_interval` | string | `500ms` |
| `livekit_signal_relay_retry_timeout` | string | `30s` |
| `livekit_signal_relay_stream_buffer_size` | string | `1000` |
| `livekit_turn_embedded_cert_file` | string | `/path/to/cert.pem` |
| `livekit_turn_embedded_domain` | string | `{{ ot_domain }}` |
| `livekit_turn_embedded_enabled` | boolean | `true` |
| `livekit_turn_embedded_external_tls` | boolean | `true` |
| `livekit_turn_embedded_key_file` | string | `/path/to/key.pem` |
| `livekit_turn_embedded_relay_range_end` | string | `30000` |
| `livekit_turn_embedded_relay_range_start` | string | `1024` |
| `livekit_turn_embedded_tls_enabled` | boolean | `false` |
| `livekit_turn_embedded_tls_port` | integer | `5349` |
| `livekit_turn_embedded_udp_port` | integer | `3478` |
| `livekit_webhook_api_key` | null | `null` |
| `livekit_webhook_urls` | list | `[https://your-host.com/handler]` |
| `secret` | null | `null` |

## opentalk_compose.mailcrab

| Variable | Type | Default Value |
|----------|------|---------------|
| `mailcrab_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `mailcrab_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `mailcrab_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `mailcrab_docker_compose_services_state` | string | `{{ compose_service_mailcrab_state \| default('present') }}` |
| `mailcrab_docker_compose_ulimits_nofile_hard` | integer | `32768` |
| `mailcrab_docker_compose_ulimits_nofile_soft` | integer | `16384` |
| `mailcrab_docker_compose_ulimits_nproc` | integer | `4096` |
| `mailcrab_docker_image_src` | string | `registry.heinlein.group/opentalk/mailcrab` |
| `mailcrab_docker_label_http_routers_minio_console_middlewares` | string | `default-auth@file` |
| `mailcrab_webui_prefix` | string | `/adm/mailcrab` |

## opentalk_compose.garage

| Variable | Type | Default Value |
|----------|------|---------------|
| `garage_admin_api_bind_addr` | string | `[::]:3903` |
| `garage_admin_api_port` | integer | `3903` |
| `garage_bucket_name` | string | `opentalk` |
| `garage_db_engine` | string | `lmdb` |
| `garage_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `garage_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `garage_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `garage_docker_compose_services_state` | string | `{{ compose_service_garage_state \| default('present') }}` |
| `garage_docker_image_src` | string | `registry.heinlein.group/opentalk/garage` |
| `garage_key_name` | string | `opentalk-key` |
| `garage_layout_capacity` | string | `1G` |
| `garage_layout_zone` | string | `dc1` |
| `garage_replication_factor` | integer | `1` |
| `garage_rpc_bind_addr` | string | `[::]:3901` |
| `garage_rpc_port` | integer | `3901` |
| `garage_rpc_public_addr` | string | `garage:3901` |
| `garage_s3_api_bind_addr` | string | `[::]:3900` |
| `garage_s3_api_port` | integer | `3900` |
| `garage_s3_region` | string | `garage` |

## opentalk_compose.minio

> **Note:** MinIO is an alternative S3 storage provider. To use MinIO instead of the default Garage, set `s3_storage_provider: minio` in `settings.yml`.

| Variable | Type | Default Value |
|----------|------|---------------|
| `minio_bucket_name` | string | `opentalk` |
| `minio_s3_api_port` | integer | `9000` | Host port bound to `127.0.0.1` for the MinIO S3 API. Used by `rclone` during S3 migration. |
| `minio_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `minio_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `minio_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `minio_docker_compose_services_state` | string | `{{ compose_service_minio_state \| default('present') }}` |
| `minio_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `minio_docker_compose_ulimits_nofile_soft` | integer | `65536` |
| `minio_docker_compose_ulimits_nproc` | integer | `8192` |
| `minio_docker_image_src` | string | `registry.heinlein.group/opentalk/minio` |
| `minio_docker_label_http_routers_minio_console_middlewares` | string | `minio-stripprefix` |
| `minio_s3_api_port` | integer | `9000` |

## opentalk_compose.s3_migrate

| Variable | Type | Default Value | Description |
|----------|------|---------------|-------------|
| `s3_migration_source` | string | — | **Required.** Source provider: `minio` or `garage`. Pass via `-e s3_migration_source=minio`. |
| `s3_migrate_run_deploy` | boolean | `true` | Run `otctl deploy` automatically after migration to stop the source service. Set to `false` to skip. |
| `s3_migrate_minio_endpoint` | string | `http://127.0.0.1:9000` | MinIO S3 endpoint reachable from the host. |
| `s3_migrate_garage_endpoint` | string | `http://127.0.0.1:3900` | Garage S3 endpoint reachable from the host. |
| `s3_migrate_bucket_name` | string | `opentalk` | Bucket to migrate. |
| `s3_migrate_settings_file` | string | `/etc/opentalk-compose/settings.yml` | Path to `settings.yml` that is updated after migration. |

## opentalk_compose.ndt

| Variable | Type | Default Value |
|----------|------|---------------|
| `compose_service_ndt_enabled` | string | `{{ ndt_enable \| default(false) }}` |
| `ndt_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `ndt_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `ndt_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `ndt_docker_compose_services_state` | string | `{{ compose_service_ndt_state \| default('present') }}` |
| `ndt_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `ndt_docker_compose_ulimits_nofile_soft` | integer | `32768` |
| `ndt_docker_compose_ulimits_nproc` | integer | `4096` |
| `ndt_docker_image_src` | string | `registry.heinlein.group/opentalk/ndt-server` |

## opentalk_compose.opentalk_controller

| Variable | Type | Default Value |
|----------|------|---------------|
| `controller_authz_synchronize_controllers` | string | `true` |
| `controller_call_in_default_country_code` | string | `DE` |
| `controller_call_in_enable_phone_mapping` | string | `false` |
| `controller_call_in_tel` | string | `03012345678` |
| `controller_database_host` | string | `controller-db` |
| `controller_database_max_connections` | string | `100` |
| `controller_database_name` | string | `controller` |
| `controller_database_password` | string | `{{ CONTROLLER_DB_PASSWORD }}` |
| `controller_database_port` | integer | `5432` |
| `controller_database_user` | string | `ot` |
| `controller_db_bind_interface` | string | `127.0.0.1` |
| `controller_db_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `controller_db_docker_compose_ulimits_nofile_soft` | integer | `65536` |
| `controller_db_docker_compose_ulimits_nproc` | integer | `8192` |
| `controller_db_host_port` | integer | `5432` |
| `controller_defaults_disabled_features` | list | `[core::call_in, integration::outlook]` |
| `controller_defaults_screen_share_requires_permission` | string | `true` |
| `controller_defaults_user_language` | string | `en-US` |
| `controller_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `controller_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `controller_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `controller_docker_compose_services_state` | string | `{{ compose_service_opentalk_controller_state \| default('present') }}` |
| `controller_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `controller_docker_compose_ulimits_nofile_soft` | integer | `65536` |
| `controller_docker_compose_ulimits_nproc` | integer | `8192` |
| `controller_docker_extra_hosts` | list | `[host.docker.internal:host-gateway]` |
| `controller_docker_image_src` | string | `registry.heinlein.group/opentalk/controller` |
| `controller_endpoints_disable_openapi` | string | `false` |
| `controller_endpoints_disable_users_find` | string | `false` |
| `controller_endpoints_disallow_custom_display_name` | string | `false` |
| `controller_endpoints_event_invite_external_email_address` | string | `true` |
| `controller_endpoints_users_find_use_kc` | string | `false` |
| `controller_etcd_urls` | list | `[etcd:2379]` |
| `controller_etherpad_api_key` | string | `{{ ETHERPAD_API_KEY }}` |
| `controller_etherpad_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/etherpad/` |
| `controller_frontend_base_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}` |
| `controller_http_addr` | string | `::0` |
| `controller_http_cors_allowed_origin` | string | `["*"]` |
| `controller_http_port` | integer | `11311` |
| `controller_http_service_api_keys` | list | `[{id: controller, secret: secret}]` |
| `controller_livekit_api_key` | string | `controller` |
| `controller_livekit_api_secret` | string | `{{ LIVEKIT_API_SECRET }}` |
| `controller_livekit_public_url` | string | `wss://{{ (ot_domain, ot_ingress_port) \| build_host }}/livekit` |
| `controller_livekit_service_url` | string | `http://host.docker.internal:7880` |
| `controller_logging_default_directives` | list | `[ERROR, opentalk=INFO, pinky_swear=OFF, rustls=WARN, mio=ERROR, lapin=WARN, execution_id=trace]` |
| `controller_logging_otlp_tracing_endpoint` | string | `http://localhost:4317` |
| `controller_logging_service_instance_id` | string | `627cc493-f310-47de-96bd-71410b7dec09` |
| `controller_logging_service_name` | string | `controller` |
| `controller_logging_service_namespace` | string | `opentalk` |
| `controller_metrics_allowlist` | list | `[127.0.0.0/24, ::ffff:0:0/96]` |
| `controller_minio_access_key` | string | `{{ GARAGE_S3_ACCESS_KEY if (s3_storage_provider \| default('garage')) == 'garage' else 'minioadmin' }}` |
| `controller_minio_bucket` | string | `opentalk` |
| `controller_minio_secret_key` | string | `{{ GARAGE_S3_SECRET_KEY if (s3_storage_provider \| default('garage')) == 'garage' else MINIO_ROOT_PASSWORD }}` |
| `controller_minio_uri` | string | `{{ 'http://garage:3900' if (s3_storage_provider \| default('garage')) == 'garage' else 'http://minio:9000' }}` |
| `controller_monitoring_addr` | string | `0.0.0.0` |
| `controller_monitoring_port` | integer | `11411` |
| `controller_oidc_authority` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` |
| `controller_oidc_controller_authority` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` |
| `controller_oidc_controller_client_id` | string | `Controller` |
| `controller_oidc_controller_client_secret` | string | `{{ OIDC_CONTROLLER_CLIENT_SECRET }}` |
| `controller_oidc_frontend_authority` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` |
| `controller_oidc_frontend_client_id` | string | `Frontend` |
| `controller_operator_information_data_protection_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/data-protection` |
| `controller_operator_information_support_email_address` | string | `support@{{ ot_domain }}` |
| `controller_operator_information_support_phone_number` | string | `+49 123 4567890` |
| `controller_rabbit_mq_mail_task_queue` | string | `opentalk_mailer` |
| `controller_rabbit_mq_max_channels_per_connection` | integer | `100` |
| `controller_rabbit_mq_min_connections` | integer | `10` |
| `controller_rabbit_mq_recording_task_queue` | string | `opentalk_recorder` |
| `controller_rabbit_mq_url` | string | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` |
| `controller_redis_url` | string | `redis://localhost:6379/` |
| `controller_reports_template_inline` | string | `\|` |
| `controller_reports_url` | string | `http://localhost:6560` |
| `controller_shared_folder_directory` | string | `opentalk/meetings` |
| `controller_shared_folder_expiry` | string | `48` |
| `controller_shared_folder_password` | string | `v3rys3cr3t` |
| `controller_shared_folder_provider` | string | `nextcloud` |
| `controller_shared_folder_url` | string | `https://nextcloud.example.org/` |
| `controller_shared_folder_username` | string | `exampleuser` |
| `controller_spacedeck_api_key` | string | `{{ SPACEDECK_API_KEY }}` |
| `controller_spacedeck_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/whiteboard/` |
| `controller_tariffs_assignment` | string | `static` |
| `controller_tariffs_static_tariff_name` | string | `OpenTalkDefaultTariff` |
| `controller_tariffs_status_mapping_default` | list | `[default]` |
| `controller_tariffs_status_mapping_downgraded` | list | `[unpaid]` |
| `controller_tariffs_status_mapping_downgraded_tariff_name` | string | `basic` |
| `controller_tariffs_status_mapping_paid` | list | `[paid, all_ok]` |
| `controller_tenants_assignment` | string | `static` |
| `controller_tenants_external_tenant_id_user_attribute_name` | string | `tenant_id` |
| `controller_tenants_static_tenant_id` | string | `OpenTalkDefaultTenant` |
| `controller_user_search_api_base_url` | string | `{{ keycloak_api_base_url }}` |
| `controller_user_search_backend` | string | `keycloak_webapi` |
| `controller_user_search_client_id` | string | `Controller` |
| `controller_user_search_client_secret` | string | `{{ OIDC_CONTROLLER_CLIENT_SECRET }}` |
| `controller_user_search_users_find_behavior` | string | `from_user_search_backend` |
| `postgres_docker_image_src` | string | `registry.heinlein.group/opentalk/postgres` |

## opentalk_compose.opentalk_obelisk

| Variable | Type | Default Value |
|----------|------|---------------|
| `obelisk_auth_client_id` | string | `Obelisk` |
| `obelisk_auth_client_secret` | string | `{{ OIDC_OBELISK_CLIENT_SECRET }}` |
| `obelisk_auth_issuer` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` |
| `obelisk_clock_format` | string | `%d.%m.%y %X %Z` |
| `obelisk_controller_domain` | string | `{{ ot_domain }}/controller` |
| `obelisk_controller_insecure` | boolean | `false` |
| `obelisk_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `obelisk_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `obelisk_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `obelisk_docker_compose_services_state` | string | `{{ compose_service_opentalk_obelisk_state \| default('present') }}` |
| `obelisk_docker_compose_ulimits_nofile_hard` | integer | `98304` |
| `obelisk_docker_compose_ulimits_nofile_soft` | integer | `98304` |
| `obelisk_docker_compose_ulimits_nproc` | integer | `16384` |
| `obelisk_docker_extra_hosts` | list | `[host.docker.internal:host-gateway]` |
| `obelisk_docker_image_src` | string | `registry.heinlein.group/opentalk/obelisk` |
| `obelisk_docker_network_mode` | string | `host` |
| `obelisk_docker_ports` | list | `[5060:5060]` |
| `obelisk_environment_gst_debug_level` | integer | `2` |
| `obelisk_environment_rust_log_level` | string | `info` |
| `obelisk_language` | string | `de` |
| `obelisk_monitoring_addr` | string | `0.0.0.0` |
| `obelisk_monitoring_enable` | boolean | `true` |
| `obelisk_monitoring_port` | integer | `11411` |
| `obelisk_sip_addr` | string | `0.0.0.0` |
| `obelisk_sip_contact` | string | `` |
| `obelisk_sip_display_name_source` | string | `` |
| `obelisk_sip_encode_video_at_half_bitrate` | string | `` |
| `obelisk_sip_enforce_qop` | boolean | `false` |
| `obelisk_sip_id` | string | `` |
| `obelisk_sip_inbound_addr` | string | `192.0.2.1:5060` |
| `obelisk_sip_inbound_media_ip` | string | `192.0.2.1` |
| `obelisk_sip_max_video_bitrate` | integer | `6000000` |
| `obelisk_sip_nat_ping_delta` | string | `` |
| `obelisk_sip_offer_srtp` | string | `` |
| `obelisk_sip_outbound_proxy` | string | `` |
| `obelisk_sip_password` | string | `` |
| `obelisk_sip_port` | integer | `5060` |
| `obelisk_sip_realm` | string | `` |
| `obelisk_sip_registrar` | string | `` |
| `obelisk_sip_rtp_port_range_end` | integer | `49999` |
| `obelisk_sip_rtp_port_range_start` | integer | `40000` |
| `obelisk_sip_stun_server` | string | `` |
| `obelisk_sip_transport` | string | `` |
| `obelisk_sip_username` | string | `` |

## opentalk_compose.opentalk_outlook_add_in

| Variable | Type | Default Value |
|----------|------|---------------|
| `outlook_add_in_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `outlook_add_in_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `outlook_add_in_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `outlook_add_in_docker_compose_services_state` | string | `{{ compose_service_opentalk_outlook_add_in_state \| default('present') }}` |
| `outlook_add_in_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `outlook_add_in_docker_compose_ulimits_nofile_soft` | integer | `65536` |
| `outlook_add_in_docker_compose_ulimits_nproc` | integer | `8192` |
| `outlook_add_in_docker_image_src` | string | `registry.heinlein.group/opentalk/outlook-add-in` |
| `outlook_add_in_labels_traefik_http_middlewares_outlook_add_in_cors_headers_accessControlAllowOriginList` | string | `{{ (ot_ingress_protocol, ot_domain, '*') \| build_url }}` |
| `outlook_add_in_oidc_client_id` | string | `OutlookAddin` |
| `outlook_add_in_oidc_scopes` | string | `profile email openid` |
| `outlook_add_in_outlook_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/outlook` |
| `outlook_add_in_outlook_webapp_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}` |

## opentalk_compose.opentalk_recorder

| Variable | Type | Default Value |
|----------|------|---------------|
| `recorder_auth_client_id` | string | `Recorder` |
| `recorder_auth_client_secret` | string | `{{ OIDC_RECORDER_CLIENT_SECRET }}` |
| `recorder_auth_issuer` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` |
| `recorder_controller_domain` | string | `controller:11311` |
| `recorder_controller_insecure` | string | `true` |
| `recorder_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `recorder_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `recorder_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `recorder_docker_compose_services_state` | string | `{{ compose_service_opentalk_recorder_state \| default('present') }}` |
| `recorder_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `recorder_docker_compose_ulimits_nofile_soft` | integer | `65536` |
| `recorder_docker_compose_ulimits_nproc` | integer | `8192` |
| `recorder_docker_extra_hosts` | list | `[host.docker.internal:host-gateway]` |
| `recorder_docker_image_src` | string | `registry.heinlein.group/opentalk/recorder` |
| `recorder_hardware_acceleration_device` | string | `/dev/dri/renderD129` |
| `recorder_hardware_acceleration_manufacturer` | string | `intel` |
| `recorder_http_api_keys` | list | `[{id: recorder, secret: secret}]` |
| `recorder_http_host` | string | `0.0.0.0` |
| `recorder_http_port` | string | `5555` |
| `recorder_monitoring_port` | integer | `11411` |
| `recorder_orchestrator_api_key_id` | string | `orchestrator` |
| `recorder_orchestrator_api_key_secret` | string | `secret` |
| `recorder_orchestrator_url` | string | `http://127.0.0.1:11222` |
| `recorder_rabbitmq_queue` | string | `opentalk_recorder` |
| `recorder_rabbitmq_uri` | string | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` |
| `recorder_recorder_clock_format` | string | `%d.%m.%y %X %Z` |
| `recorder_recorder_display` | string | `true` |

## opentalk_compose.opentalk_smtp_mailer

| Variable | Type | Default Value |
|----------|------|---------------|
| `mailer_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `mailer_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `mailer_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `mailer_docker_compose_services_state` | string | `{{ compose_service_opentalk_smtp_mailer_state \| default('present') }}` |
| `mailer_docker_compose_ulimits_nofile_hard` | integer | `32768` |
| `mailer_docker_compose_ulimits_nofile_soft` | integer | `16384` |
| `mailer_docker_compose_ulimits_nproc` | integer | `4096` |
| `mailer_docker_image_src` | string | `registry.heinlein.group/opentalk/smtp-mailer` |
| `mailer_frontend_base_url` | string | `https://{{ ot_domain }}` |
| `mailer_frontend_data_protection_url` | string | `https://opentalk.example.org/dataprotection` |
| `mailer_languages_default_language` | string | `de-DE` |
| `mailer_monitoring_addr` | string | `0.0.0.0` |
| `mailer_monitoring_port` | integer | `11411` |
| `mailer_rabbit_mq_mail_task_queue` | string | `opentalk_mailer` |
| `mailer_rabbit_mq_task_processing_timeout_seconds` | string | `1200` |
| `mailer_rabbit_mq_url` | string | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` |
| `mailer_smtp_from_email` | string | `no-reply@{{ ot_domain }}` |
| `mailer_smtp_from_name` | string | `OpenTalk - {{ ot_domain }}` |
| `mailer_smtp_smtp_server` | string | `smtp://mailcrab:1025?disable_starttls=true` |
| `mailer_support_contact_mail` | string | `support@example.com` |
| `mailer_support_contact_phone` | string | `+49 123 456789` |
| `mailer_template_builder_dashboard_event_link_builder` | string | `{base_url}/dashboard/meetings/view/{event_id}` |
| `mailer_template_builder_guest_link_builder` | string | `{base_url}/invite/{invite_id}` |
| `mailer_template_builder_join_link_builder` | string | `{base_url}lobby/{room_id}` |
| `mailer_templates_external_event_cancellation` | string | `{html = "resources/templates/external_event_cancellation.html", txt = "resources/templates/external_event_cancellation.txt"}` |
| `mailer_templates_external_event_update` | string | `{html = "resources/templates/external_event_update.html", txt = "resources/templates/external_event_update.txt"}` |
| `mailer_templates_external_invite` | string | `{html = "resources/templates/external_invite.html", txt = "resources/templates/external_invite.txt"}` |
| `mailer_templates_external_uninvite` | string | `{html = "resources/templates/external_uninvite.html", txt = "resources/templates/external_uninvite.txt"}` |
| `mailer_templates_registered_event_cancellation` | string | `{html = "resources/templates/registered_event_cancellation.html", txt = "resources/templates/registered_event_cancellation.txt"}` |
| `mailer_templates_registered_event_update` | string | `{html = "resources/templates/registered_event_update.html", txt = "resources/templates/registered_event_update.txt"}` |
| `mailer_templates_registered_invite` | string | `{html = "resources/templates/registered_invite.html", txt = "resources/templates/registered_invite.txt"}` |
| `mailer_templates_registered_uninvite` | string | `{html = "resources/templates/registered_uninvite.html", txt = "resources/templates/registered_uninvite.txt"}` |
| `mailer_templates_unregistered_event_cancellation` | string | `{html = "resources/templates/unregistered_event_cancellation.html", txt = "resources/templates/unregistered_event_cancellation.txt"}` |
| `mailer_templates_unregistered_event_update` | string | `{html = "resources/templates/unregistered_event_update.html", txt = "resources/templates/unregistered_event_update.txt"}` |
| `mailer_templates_unregistered_invite` | string | `{html = "resources/templates/unregistered_invite.html", txt = "resources/templates/unregistered_invite.txt"}` |
| `mailer_templates_unregistered_uninvite` | string | `{html = "resources/templates/unregistered_uninvite.html", txt = "resources/templates/unregistered_uninvite.txt"}` |

## opentalk_compose.opentalk_web_frontend

| Variable | Type | Default Value |
|----------|------|---------------|
| `opentalk_web_frontend_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `opentalk_web_frontend_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `opentalk_web_frontend_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `opentalk_web_frontend_docker_compose_services_state` | string | `{{ compose_service_opentalk_web_frontend_state \| default('present') }}` |
| `opentalk_web_frontend_docker_compose_ulimits_nofile_hard` | integer | `8192` |
| `opentalk_web_frontend_docker_compose_ulimits_nofile_soft` | integer | `4096` |
| `opentalk_web_frontend_docker_compose_ulimits_nproc` | integer | `2048` |
| `opentalk_web_frontend_docker_extra_hosts` | list | `[]` |
| `opentalk_web_frontend_docker_image_src` | string | `registry.heinlein.group/opentalk/web-frontend` |
| `opentalk_web_frontend_environment_base_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}` |
| `opentalk_web_frontend_environment_change_password_url` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}/account/` |
| `opentalk_web_frontend_environment_controller_host` | string | `{{ (ot_domain, ot_ingress_port) \| build_host }}/controller` |
| `opentalk_web_frontend_environment_error_report_address` | string | `{{ ERROR_REPORT_ADDRESS \| default('reports@example.com') }}` |
| `opentalk_web_frontend_environment_feature_e2e_encryption` | boolean | `false` |
| `opentalk_web_frontend_environment_feature_protocol` | string | `{{ FEATURE_PROTOCOL \| default(true) }}` |
| `opentalk_web_frontend_environment_feature_recording` | string | `{{ FEATURE_RECORDING \| default(false) }}` |
| `opentalk_web_frontend_environment_feature_timer` | string | `{{ FEATURE_TIMER \| default(true) }}` |
| `opentalk_web_frontend_environment_feature_user_search` | string | `{{ FEATURE_USER_SEARCH \| default(true) }}` |
| `opentalk_web_frontend_environment_feature_whiteboard` | string | `{{ FEATURE_WHITEBOARD \| default(true) }}` |
| `opentalk_web_frontend_environment_insecure` | boolean | `false` |
| `opentalk_web_frontend_environment_is_beta_release` | string | `{{ IS_BETA_RELEASE \| default(false) }}` |
| `opentalk_web_frontend_environment_libravatar_default_image` | string | `{{ LIBRAVATAR_DEFAULT_IMAGE \| default('identicon') }}` |
| `opentalk_web_frontend_environment_livekit_e2ee_salt` | string | `{{ LIVEKIT_E2EE_SALT }}` |
| `opentalk_web_frontend_environment_livekit_preferred_video_codec` | string | `{{ LIVEKIT_PREFERRED_VIDEO_CODEC \| default('vp9') }}` |
| `opentalk_web_frontend_environment_livekit_server_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/livekit` |
| `opentalk_web_frontend_environment_ndt_server` | string | `{{ (ot_domain, ot_ingress_port) \| build_host }}` |
| `opentalk_web_frontend_environment_oidc_client_id` | string | `{{ controller_oidc_frontend_client_id \| default('Frontend') }}` |
| `opentalk_web_frontend_environment_oidc_issuer` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` |
| `opentalk_web_frontend_environment_suppress_browser_compatibility_info` | boolean | `false` |
| `opentalk_web_frontend_environment_video_backgrounds` | string | undefined (commented out). When defined, backgrounds directory is mounted as volume. Example: `[{ altText: 'My BG', url: '/assets/videoBackgrounds/my-bg.png', thumb: '/assets/videoBackgrounds/thumbs/my-bg-thumb.png' }]` |
| `opentalk_web_frontend_backgrounds_dir` | string | `/var/opt/opentalk-compose/backgrounds` |
| `opentalk_web_frontend_environment_waiting_room_default_value` | string | `{{ WAITING_ROOM_DEFAULT_VALUE \| default(false) }}` |
| `opentalk_web_frontend_labels_traefik_enable` | boolean | `true` |
| `opentalk_web_frontend_labels_traefik_http_middlewares_webapp_cors_headers_accessControlAllowOriginList` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}` |

## opentalk_compose.rabbitmq

| Variable | Type | Default Value |
|----------|------|---------------|
| `rabbitmq_default_pass` | string | `{{ RABBITMQ_PASSWORD }}` |
| `rabbitmq_default_user` | string | `ot` |
| `rabbitmq_default_vhost` | string | `/` |
| `rabbitmq_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `rabbitmq_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `rabbitmq_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `rabbitmq_docker_compose_services_state` | string | `{{ compose_service_rabbitmq_state \| default('present') }}` |
| `rabbitmq_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `rabbitmq_docker_compose_ulimits_nofile_soft` | integer | `65536` |
| `rabbitmq_docker_compose_ulimits_nproc` | integer | `8192` |
| `rabbitmq_docker_image_src` | string | `registry.heinlein.group/opentalk/rabbitmq` |
| `rabbitmq_docker_ports` | list | `[]` |
| `rabbitmq_management_path_prefix` | string | `/adm/rabbitmq` |

## opentalk_compose.s3_migrate

| Variable | Type | Default Value |
|----------|------|---------------|
| `s3_migrate_bucket_name` | string | `opentalk` |
| `s3_migrate_garage_endpoint` | string | `http://127.0.0.1:3900` |
| `s3_migrate_minio_endpoint` | string | `http://127.0.0.1:9000` |
| `s3_migrate_run_deploy` | boolean | `true` |
| `s3_migrate_settings_file` | string | `{{ config_dir \| default('/etc/opentalk-compose') }}/settings.yml` |

## opentalk_compose.secrets

| Variable | Type | Default Value |
|----------|------|---------------|
| `force_generate_secrets` | boolean | `false` |
| `no_secret_log` | boolean | `true` |

## opentalk_compose.self_signed_cert

| Variable | Type | Default Value |
|----------|------|---------------|
| `custom_certificates_ca_root_cert_file` | string | `ca.crt` |
| `custom_certificates_certs_dir` | string | `/etc/opentalk-compose/ssl` |
| `custom_certificates_enabled` | string | `false` |
| `custom_certificates_fullchain_cert_file` | string | `fullchain.crt` |
| `custom_certificates_private_key_file` | string | `privkey.key` |
| `self_signed_certificates_ca_certificate_validity_days` | integer | `365` |
| `self_signed_certificates_ca_common_name` | string | `{{ ot_domain }}` |
| `self_signed_certificates_ca_country` | string | `DE` |
| `self_signed_certificates_ca_dir` | string | `{{ custom_certificates_certs_dir }}/ca` |
| `self_signed_certificates_ca_email_address` | string | `admin@{{ ot_domain }}` |
| `self_signed_certificates_ca_key_size` | integer | `4096` |
| `self_signed_certificates_ca_locality` | string | `Berlin` |
| `self_signed_certificates_ca_organization` | string | `my CA` |
| `self_signed_certificates_ca_organizational_unit` | string | `CA` |
| `self_signed_certificates_ca_state` | string | `Berlin` |
| `self_signed_certificates_generate` | boolean | `false` |
| `self_signed_certificates_install_system_trust` | boolean | `true` |
| `self_signed_certificates_server_certificate_validity_days` | integer | `365` |
| `self_signed_certificates_server_common_name` | string | `{{ ot_domain }}` |
| `self_signed_certificates_server_country` | string | `DE` |
| `self_signed_certificates_server_email_address` | string | `admin@{{ ot_domain }}` |
| `self_signed_certificates_server_key_size` | integer | `4096` |
| `self_signed_certificates_server_locality` | string | `Berlin` |
| `self_signed_certificates_server_organization` | string | `my Company` |
| `self_signed_certificates_server_organizational_unit` | string | `IT Security` |
| `self_signed_certificates_server_san_dns` | list | `[{{ ot_domain }}, *.{{ ot_domain }}]` |
| `self_signed_certificates_server_san_ip` | list | `[127.0.0.1, ::1]` |
| `self_signed_certificates_server_state` | string | `Berlin` |
| `self_signed_certificates_ssl_certificate_ca_file` | string | `{{ custom_certificates_ca_root_cert_file \| default('ca.crt') }}` |
| `self_signed_certificates_ssl_certificate_fullchain_file` | string | `{{ custom_certificates_ssl_certificate_fullchain_file \| default('fullchain.crt') }}` |
| `self_signed_certificates_ssl_certificate_key_file` | string | `{{ custom_certificates_ssl_certificate_key_file \| default('privkey.key') }}` |
| `self_signed_certificates_system_trust_dest_override` | string | `` |
| `self_signed_certificates_system_trust_name` | string | `opentalk-selfsigned-ca.crt` |

## opentalk_compose.spacedeck

| Variable | Type | Default Value |
|----------|------|---------------|
| `spacedeck_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `spacedeck_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `spacedeck_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `spacedeck_docker_compose_services_state` | string | `{{ compose_service_spacedeck_state \| default('present') }}` |
| `spacedeck_docker_compose_ulimits_nofile_hard` | integer | `32768` |
| `spacedeck_docker_compose_ulimits_nofile_soft` | integer | `16384` |
| `spacedeck_docker_compose_ulimits_nproc` | integer | `4096` |
| `spacedeck_docker_environment_prefix` | string | `/whiteboard` |
| `spacedeck_docker_environment_sd_api_token` | string | `/run/secrets/spacedeck_api_secret` |
| `spacedeck_docker_environment_sd_endpoint` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}` |
| `spacedeck_docker_environment_sd_host` | string | `0.0.0.0` |
| `spacedeck_docker_environment_sd_invite_code` | string | `/run/secrets/spacedeck_invite_secret` |
| `spacedeck_docker_environment_sd_port` | integer | `9666` |
| `spacedeck_docker_extra_hosts` | list | `[]` |
| `spacedeck_docker_image_src` | string | `registry.heinlein.group/opentalk/spacedeck` |
| `spacedeck_docker_labels_traefik_enable` | boolean | `true` |
| `spacedeck_docker_ports` | list | `[]` |

## opentalk_compose.traefik

| Variable | Type | Default Value |
|----------|------|---------------|
| `traefik_accessLog_bufferingSize` | integer | `100` |
| `traefik_api_dashboard` | boolean | `true` |
| `traefik_api_insecure` | boolean | `false` |
| `traefik_basicAuth_dashboard_admin` | string | `admin` |
| `traefik_basicAuth_dashboard_admin_password` | string | `{{ TRAEFIK_DASHBOARD_ADMIN_PASSWORD }}` |
| `traefik_basicAuth_dashboard_admin_password_salt` | string | `{{ TRAEFIK_DASHBOARD_ADMIN_PASSWORD_SALT }}` |
| `traefik_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `traefik_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `traefik_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `traefik_docker_compose_services_state` | string | `{{ compose_service_traefik_state \| default('present') }}` |
| `traefik_docker_compose_ulimits_nofile_hard` | integer | `65536` |
| `traefik_docker_compose_ulimits_nofile_soft` | integer | `65536` |
| `traefik_docker_compose_ulimits_nproc` | integer | `8192` |
| `traefik_docker_extra_hosts` | list | `[host.docker.internal:host-gateway]` |
| `traefik_docker_image_src` | string | `registry.heinlein.group/opentalk/traefik` |
| `traefik_docker_label_http_routers_dashboard_middlewares` | string | `dashboard-strip, default-auth@file` |
| `traefik_docker_ports` | list | `[{target: 80, published: 80, mode: host}, {target: 443, published: 443, mode: host}, {target: 8080, published: 7777}]` |
| `traefik_docker_restart` | string | `always` |
| `traefik_entryPoints_ping_address` | string | `:8082` |
| `traefik_entryPoints_web_address` | string | `:80` |
| `traefik_entryPoints_websecure_address` | string | `:443` |
| `traefik_entryPoints_websecure_http_tls` | dictionary | `{}` |
| `traefik_global_checkNewVersion` | boolean | `true` |
| `traefik_global_sendAnonymousUsage` | boolean | `false` |
| `traefik_letsencrypt_acme_caServer` | string | `https://acme-v02.api.letsencrypt.org/directory` |
| `traefik_letsencrypt_acme_email` | string | `ops@{{ ot_domain }}` |
| `traefik_letsencrypt_acme_httpChallenge_entryPoint` | string | `web` |
| `traefik_letsencrypt_acme_storage` | string | `/data/acme.json` |
| `traefik_letsencrypt_enable` | boolean | `true` |
| `traefik_log_level` | string | `INFO` |
| `traefik_ping_entryPoint` | string | `ping` |
| `traefik_providers_docker_exposedByDefault` | boolean | `false` |
| `traefik_providers_file_directory` | string | `/etc/traefik/dynamic` |
| `traefik_providers_file_watch` | boolean | `true` |
| `csp_allowed_extra_img_src` | list | `[]` — Additional sources for `img-src` in the CSP header, e.g. `[https://avatars.example.com]` |
| `csp_allowed_extra_connect_src` | list | `[]` — Additional sources for `connect-src` in the CSP header, e.g. `[https://keycloak.example.com]` |
