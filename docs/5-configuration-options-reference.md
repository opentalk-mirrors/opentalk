<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Configuration Reference (Auto-generated)

Last updated: 2025-12-11

This document lists the configuration variables for each Ansible role.
Variables are collected from `defaults/*.yml` and `vars/*.yml`.
Default values should be overridden in `/etc/opentalk-compose/settings.yml`.

## opentalk_compose.backup

| Variable | Type | Default Value |
|----------|------|---------------|
| `backup_dir` | string | `/var/opt/opentalk-compose/backups` |
| `backup_dir_config` | string | `{{ backup_dir }}/config` |
| `backup_dir_sql_dumps` | string | `{{ backup_dir }}/dumps` |
| `backup_dir_volumes` | string | `{{ backup_dir }}/volumes` |
| `backup_docker_volume_list` | list | `['{{ compose_project_name }}_minio', '{{ compose_project_name }}_rabbitmq', '{{ compose_project_name }}_traefik']` |
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
| `etherpad_docker_environment_default_pad_text` | string | `Welcome to OpenTalk` |
| `etherpad_docker_environment_suppress_errors_in_pad_text` | boolean | `true` |
| `etherpad_docker_environment_trust_proxy` | boolean | `true` |
| `etherpad_docker_extra_hosts` | list | `[]` |
| `etherpad_docker_image_src` | string | `registry.opencode.de/opentalk/etherpad` |
| `etherpad_docker_labels_traefik_enable` | boolean | `true` |
| `etherpad_docker_labels_traefik_http_routers_etherpad_rule` | string | `Host(`{{ ot_domain }}`) && PathPrefix(`/etherpad`)` |
| `etherpad_docker_ports` | list | `[]` |

## opentalk_compose.keycloak

| Variable | Type | Default Value |
|----------|------|---------------|
| `keycloak_account_theme` | string | `opentalk` |
| `keycloak_api_base_url` | string | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) \| build_subdomain_url }}/auth/admin/realms/{{ keycloak_realm }}` |
| `keycloak_auth_password` | string | `{{ keycloak_permanent_admin_password }}` |
| `keycloak_auth_realm` | string | `master` |
| `keycloak_auth_username` | string | `{{ keycloak_permanent_admin_username }}` |
| `keycloak_base_url` | string | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) \| build_subdomain_url }}/auth` |
| `keycloak_bootstrap_admin_password` | string | `{{ KC_BOOTSTRAP_ADMIN_PASSWORD }}` |
| `keycloak_bootstrap_admin_remove` | boolean | `true` |
| `keycloak_bootstrap_admin_username` | string | `bootstrap_admin` |
| `keycloak_db_password` | string | `{{ KEYCLOAK_DB_PASSWORD }}` |
| `keycloak_db_schema` | string | `public` |
| `keycloak_db_url_database` | string | `keycloak` |
| `keycloak_db_url_host` | string | `keycloak-db` |
| `keycloak_db_url_port` | integer | `5432` |
| `keycloak_db_username` | string | `ot` |
| `keycloak_db_vendor` | string | `postgres` |
| `keycloak_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `keycloak_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `keycloak_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `keycloak_docker_compose_services_state` | string | `{{ compose_service_keycloak_state \| default('present') }}` |
| `keycloak_docker_compose_state` | string | `present` |
| `keycloak_docker_image_src` | string | `quay.io/keycloak/keycloak` |
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
| `keycloak_redirectUris_uris` | list | `['{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/auth/popup_callback', '{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/auth/callback', '{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/dashboard', '{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/dashboard/*', '{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/room/*', '{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/join', '{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/']` |
| `keycloak_spi_sticky_session_encoder_infinispan_should_attach_route` | string | `false` |
| `keycloak_wait_to_be_reachable_delay` | integer | `10` |
| `keycloak_wait_to_be_reachable_retries` | integer | `20` |
| `postInstall_demoUser_email` | string | `demo.user@example.com` |
| `postInstall_demoUser_enabled` | string | `True` |
| `postInstall_demoUser_firstName` | string | `Test` |
| `postInstall_demoUser_lastName` | string | `User` |
| `postInstall_demoUser_name` | string | `demo` |
| `postInstall_demoUser_password` | string | `{{ DEMO_USER_PASSWORD }}` |
| `postInstall_demoUser_state` | string | `present` |
| `postgres_docker_image_src` | string | `postgres` |

## opentalk_compose.livekit_server

| Variable | Type | Default Value |
|----------|------|---------------|
| `livekit_api_key_controller` | string | `{{ LIVEKIT_API_SECRET }}` |
| `livekit_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `livekit_docker_image_src` | string | `livekit/livekit-server` |
| `livekit_port` | integer | `7880` |
| `livekit_rtc_enable_loopback_candidate` | boolean | `false` |
| `livekit_rtc_ips_excludes` | list | `['10.0.0.0/8', '172.16.0.0/12', '192.168.0.0/16']` |
| `livekit_rtc_ips_includes` | list | `[]` |
| `livekit_rtc_port_range_end` | integer | `64535` |
| `livekit_rtc_port_range_start` | integer | `50000` |
| `livekit_rtc_tcp_port` | integer | `7881` |
| `livekit_rtc_use_external_ip` | boolean | `true` |
| `livekit_server_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `livekit_server_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `livekit_server_docker_compose_services_state` | string | `{{ compose_service_livekit_server_state \| default('present') }}` |
| `livekit_turn_embedded_domain` | string | `{{ ot_domain }}` |
| `livekit_turn_embedded_enabled` | boolean | `true` |
| `livekit_turn_embedded_external_tls` | boolean | `true` |
| `livekit_turn_embedded_tls_enabled` | boolean | `false` |
| `livekit_turn_embedded_tls_port` | integer | `5349` |
| `livekit_turn_embedded_udp_port` | integer | `3478` |

## opentalk_compose.mailcrab

| Variable | Type | Default Value |
|----------|------|---------------|
| `mailcrab_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `mailcrab_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `mailcrab_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `mailcrab_docker_compose_services_state` | string | `{{ compose_service_mailcrab_state \| default('present') }}` |
| `mailcrab_docker_image_src` | string | `marlonb/mailcrab` |
| `mailcrab_docker_label_http_routers_minio_console_middlewares` | string | `default-auth@file` |
| `mailcrab_webui_prefix` | string | `/adm/mailcrab` |

## opentalk_compose.minio

| Variable | Type | Default Value |
|----------|------|---------------|
| `minio_bucket_name` | string | `opentalk` |
| `minio_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `minio_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `minio_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `minio_docker_compose_services_state` | string | `{{ compose_service_minio_state \| default('present') }}` |
| `minio_docker_image_src` | string | `minio/minio` |
| `minio_docker_label_http_routers_minio_console_middlewares` | string | `minio-stripprefix` |

## opentalk_compose.ndt

| Variable | Type | Default Value |
|----------|------|---------------|
| `compose_service_ndt_enabled` | string | `{{ ndt_enable \| default(false) }}` |
| `ndt_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `ndt_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `ndt_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `ndt_docker_compose_services_state` | string | `{{ compose_service_ndt_state \| default('present') }}` |
| `ndt_docker_image_src` | string | `measurementlab/ndt-server` |

## opentalk_compose.opentalk_controller

| Variable | Type | Default Value |
|----------|------|---------------|
| `controller_database_host` | string | `controller-db` |
| `controller_database_name` | string | `controller` |
| `controller_database_password` | string | `{{ CONTROLLER_DB_PASSWORD }}` |
| `controller_database_port` | integer | `5432` |
| `controller_database_user` | string | `ot` |
| `controller_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `controller_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `controller_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `controller_docker_compose_services_state` | string | `{{ compose_service_opentalk_controller_state \| default('present') }}` |
| `controller_docker_extra_hosts` | list | `['host.docker.internal:host-gateway']` |
| `controller_docker_image_src` | string | `registry.opencode.de/opentalk/controller` |
| `controller_endpoints_event_invite_external_email_address` | string | `true` |
| `controller_etherpad_api_key` | string | `{{ ETHERPAD_API_KEY }}` |
| `controller_etherpad_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/etherpad/` |
| `controller_frontend_base_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}` |
| `controller_http_port` | integer | `11311` |
| `controller_livekit_api_key` | string | `controller` |
| `controller_livekit_api_secret` | string | `{{ LIVEKIT_API_SECRET }}` |
| `controller_livekit_public_url` | string | `wss://{{ (ot_domain, ot_ingress_port) \| build_host }}/livekit` |
| `controller_livekit_service_url` | string | `http://host.docker.internal:7880` |
| `controller_logging_default_directives` | list | `['ERROR', 'opentalk=INFO', 'pinky_swear=OFF', 'rustls=WARN', 'mio=ERROR', 'lapin=WARN', 'execution_id=trace']` |
| `controller_minio_access_key` | string | `minioadmin` |
| `controller_minio_bucket` | string | `opentalk` |
| `controller_minio_secret_key` | string | `{{ MINIO_ROOT_PASSWORD }}` |
| `controller_minio_uri` | string | `http://minio:9000` |
| `controller_monitoring_port` | integer | `11411` |
| `controller_oidc_authority` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` |
| `controller_oidc_controller_client_id` | string | `Controller` |
| `controller_oidc_controller_client_secret` | string | `{{ OIDC_CONTROLLER_CLIENT_SECRET }}` |
| `controller_oidc_frontend_client_id` | string | `Frontend` |
| `controller_operator_information_data_protection_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/data-protection` |
| `controller_operator_information_support_email_address` | string | `support@{{ ot_domain }}` |
| `controller_operator_information_support_phone_number` | string | `+49 123 4567890` |
| `controller_rabbit_mq_mail_task_queue` | string | `opentalk_mailer` |
| `controller_rabbit_mq_max_channels_per_connection` | integer | `100` |
| `controller_rabbit_mq_min_connections` | integer | `10` |
| `controller_rabbit_mq_recording_task_queue` | string | `opentalk_recorder` |
| `controller_rabbit_mq_url` | string | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` |
| `controller_spacedeck_api_key` | string | `{{ SPACEDECK_API_KEY }}` |
| `controller_spacedeck_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/whiteboard/` |
| `controller_user_search_api_base_url` | string | `{{ keycloak_api_base_url }}` |
| `controller_user_search_backend` | string | `keycloak_webapi` |
| `controller_user_search_users_find_behavior` | string | `from_user_search_backend` |
| `postgres_docker_image_src` | string | `postgres` |

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
| `obelisk_docker_extra_hosts` | list | `['host.docker.internal:host-gateway']` |
| `obelisk_docker_image_src` | string | `registry.opencode.de/opentalk/obelisk` |
| `obelisk_docker_network_mode` | string | `` |
| `obelisk_docker_ports` | list | `['5060:5060']` |
| `obelisk_environment_gst_debug_level` | integer | `2` |
| `obelisk_environment_rust_log_level` | string | `info` |
| `obelisk_monitoring_addr` | string | `0.0.0.0` |
| `obelisk_monitoring_enable` | boolean | `true` |
| `obelisk_monitoring_port` | integer | `11411` |
| `obelisk_sip_addr` | string | `0.0.0.0` |
| `obelisk_sip_contact` | string | `` |
| `obelisk_sip_enforce_qop` | boolean | `true` |
| `obelisk_sip_id` | string | `` |
| `obelisk_sip_max_video_bitrate` | integer | `6000` |
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
| `obelisk_sip_username` | string | `` |

## opentalk_compose.opentalk_outlook_add_in

| Variable | Type | Default Value |
|----------|------|---------------|
| `outlook_add_in_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `outlook_add_in_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `outlook_add_in_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `outlook_add_in_docker_compose_services_state` | string | `{{ compose_service_opentalk_outlook_add_in_state \| default('present') }}` |
| `outlook_add_in_docker_image_src` | string | `registry.opencode.de/opentalk/opentalk-outlook-add-in` |
| `outlook_add_in_labels_traefik_http_middlewares_outlook_add_in_cors_headers_accessControlAllowOriginList` | string | `{{ (ot_ingress_protocol, ot_domain, '*') \| build_url }}` |
| `outlook_add_in_oidc_client_id` | string | `OutlookAddin` |
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
| `recorder_docker_extra_hosts` | list | `['host.docker.internal:host-gateway']` |
| `recorder_docker_image_src` | string | `registry.opencode.de/opentalk/recorder` |
| `recorder_monitoring_port` | integer | `11411` |
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
| `mailer_docker_image_src` | string | `registry.opencode.de/opentalk/smtp-mailer` |
| `mailer_frontend_base_url` | string | `https://{{ ot_domain }}` |
| `mailer_languages_default_language` | string | `de-DE` |
| `mailer_monitoring_addr` | string | `0.0.0.0` |
| `mailer_monitoring_port` | integer | `11411` |
| `mailer_rabbit_mq_mail_task_queue` | string | `opentalk_mailer` |
| `mailer_rabbit_mq_url` | string | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` |
| `mailer_smtp_from_email` | string | `no-reply@{{ ot_domain }}` |
| `mailer_smtp_from_name` | string | `OpenTalk - {{ ot_domain }}` |
| `mailer_smtp_smtp_server` | string | `smtp://mailcrab:1025?disable_starttls=true` |
| `mailer_support_contact_mail` | string | `support@{{ ot_domain }}` |
| `mailer_support_contact_phone` | string | `+49 123 456789` |

## opentalk_compose.opentalk_web_frontend

| Variable | Type | Default Value |
|----------|------|---------------|
| `opentalk_web_frontend_docker_compose_project_name` | string | `{{ compose_project_name \| default('opentalk') }}` |
| `opentalk_web_frontend_docker_compose_project_src` | string | `{{ deploy_dir }}` |
| `opentalk_web_frontend_docker_compose_services_dir` | string | `{{ services_dir }}` |
| `opentalk_web_frontend_docker_compose_services_state` | string | `{{ compose_service_opentalk_web_frontend_state \| default('present') }}` |
| `opentalk_web_frontend_docker_extra_hosts` | list | `[]` |
| `opentalk_web_frontend_docker_image_src` | string | `registry.opencode.de/opentalk/web-frontend` |
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
| `opentalk_web_frontend_environment_livekit_server_url` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}/livekit` |
| `opentalk_web_frontend_environment_ndt_server` | string | `{{ (ot_domain, ot_ingress_port) \| build_host }}` |
| `opentalk_web_frontend_environment_oidc_client_id` | string | `{{ controller_oidc_frontend_client_id \| default('Frontend') }}` |
| `opentalk_web_frontend_environment_oidc_issuer` | string | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` |
| `opentalk_web_frontend_environment_suppress_browser_compatibility_info` | boolean | `false` |
| `opentalk_web_frontend_environment_video_backgrounds` | string | `[{`<br>`  altText: 'OpenTalk',`<br>`  url: '/assets/videoBackgrounds/elevate-bg.png',`<br>`  thumb: '/assets/videoBackgrounds/thumbs/elevate-bg-thumb.png',`<br>`}]` |
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
| `rabbitmq_docker_image_src` | string | `rabbitmq` |
| `rabbitmq_docker_ports` | list | `[]` |
| `rabbitmq_management_path_prefix` | string | `/adm/rabbitmq` |

## opentalk_compose.secrets

| Variable | Type | Default Value |
|----------|------|---------------|
| `force_generate_secrets` | boolean | `false` |
| `no_secret_log` | boolean | `true` |

## opentalk_compose.self_signed_cert

| Variable | Type | Default Value |
|----------|------|---------------|
| `custom_certificates_certs_dir` | string | `/etc/opentalk-compose/ssl` |
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
| `self_signed_certificates_server_san_dns` | list | `['{{ ot_domain }}', '*.{{ ot_domain }}']` |
| `self_signed_certificates_server_san_ip` | list | `['127.0.0.1', '::1']` |
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
| `spacedeck_docker_environment_prefix` | string | `/whiteboard` |
| `spacedeck_docker_environment_sd_api_token` | string | `/run/secrets/spacedeck_api_secret` |
| `spacedeck_docker_environment_sd_endpoint` | string | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) \| build_url }}` |
| `spacedeck_docker_environment_sd_host` | string | `0.0.0.0` |
| `spacedeck_docker_environment_sd_invite_code` | string | `/run/secrets/spacedeck_invite_secret` |
| `spacedeck_docker_environment_sd_port` | integer | `9666` |
| `spacedeck_docker_extra_hosts` | list | `[]` |
| `spacedeck_docker_image_src` | string | `registry.opencode.de/opentalk/spacedeck` |
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
| `traefik_docker_extra_hosts` | list | `['host.docker.internal:host-gateway']` |
| `traefik_docker_image_src` | string | `traefik` |
| `traefik_docker_label_http_routers_dashboard_middlewares` | string | `dashboard-strip, default-auth@file` |
| `traefik_docker_ports` | list | `[{'target': 80, 'published': 80, 'mode': 'host'}, {'target': 443, 'published': 443, 'mode': 'host'}, {'target': 8080, 'published': 7777}]` |
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
