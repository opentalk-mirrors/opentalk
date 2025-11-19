<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# OpenTalk configuration options reference

> This page is being regenerated. The tables below will be replaced by an automated extract of current Ansible role defaults. Some legacy parameters may disappear if they are no longer present.

Notes:

- Dot notation denotes nesting (section.subkey.subkey).
- Defaults show literal values from defaults; Jinja placeholders like {{ VAR }} are filled from settings/secrets at render time.

## Overview (Auto-generated)

Run extras/scripts/generate-config-reference.py to refresh.

Template-only variables: referenced in Jinja2 templates but lacking a default; these may need explicit definition in settings.yml.

### opentalk_compose.backup

| Variable | Type | Default | Description |
|---|---|---|---|
| backup_dir | string | `/var/opt/opentalk-compose/backups` | Backup dir. |
| backup_dir_config | derived | `{{ backup_dir }}/config` | Backup dir config. |
| backup_dir_sql_dumps | derived | `{{ backup_dir }}/dumps` | Backup dir sql dumps. |
| backup_dir_volumes | derived | `{{ backup_dir }}/volumes` | Backup dir volumes. |
| backup_docker_volume_list | list[str] | {{ compose_project_name }}_minio<br>{{ compose_project_name }}_rabbitmq<br>{{ compose_project_name }}_traefik | Backup docker volume list. |
| backup_docker_volume_restore_files | list | `[]` | Backup docker volume restore files. |
| backup_retention_max_age_days | int | `7` | Backup retention max age days. |
| backup_retention_max_items | int | `5` | Backup retention max items. |
| config_backup_file_name | derived | `opentalk-compose_config_backup-{{ lookup('pipe','date +%Y%m%d-%H%M%S') }}.tar.gz` | Config backup file name. |
| config_backup_restore_file | string | `` | Config backup restore file. |
| controller_db_dump_file | derived | `{{ backup_dir_sql_dumps }}/controller-{{ lookup('pipe','date +%Y%m%d-%H%M%S') }}.sql` | Controller db dump file. |
| controller_db_dump_restore_file | string | `` | Controller db dump restore file. |
| controller_db_host | derived | `{{ controller_database_host | default('controller-db') }}` | Controller db host. |
| controller_db_name | derived | `{{ controller_database_name | default('controller') }}` | Controller db name. |
| controller_db_password | secret[derived] | `{{ CONTROLLER_DB_PASSWORD }}` | Controller db password. |
| controller_db_port | derived | `{{ controller_database_port | default(5432) }}` | Controller db port. |
| controller_db_scheduled_backup_enabled | bool | `True` | Controller db scheduled backup enabled. |
| controller_db_scheduled_backup_time | string | `02:00` | Controller db scheduled backup time. |
| controller_db_user | derived | `{{ controller_database_user | default('ot') }}` | Controller db user. |
| keycloak_db_dump_file | derived | `{{ backup_dir_sql_dumps }}/keycloak-{{ lookup('pipe','date +%Y%m%d-%H%M%S') }}.sql` | Keycloak db dump file. |
| keycloak_db_host | derived | `{{ keycloak_database_host | default('keycloak-db') }}` | Keycloak db host. |
| keycloak_db_name | derived | `{{ keycloak_database_name | default('keycloak') }}` | Keycloak db name. |
| keycloak_db_password | secret[derived] | `{{ KEYCLOAK_DB_PASSWORD }}` | Keycloak db password. |
| keycloak_db_port | derived | `{{ keycloak_database_port | default(5432) }}` | Keycloak db port. |
| keycloak_db_user | derived | `{{ keycloak_database_user | default('ot') }}` | Keycloak db user. |

### opentalk_compose.docker_env

| Variable | Type | Default | Description |
|---|---|---|---|
| docker_network_driver | string | `bridge` | Docker network driver. |
| docker_network_name | string | `opentalk_internal` | Docker network name. |
| docker_network_state | string | `present` | Docker network state. |
| docker_overwrite_main_compose_file | bool | `False` | Docker overwrite main  file. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |

### opentalk_compose.etherpad

| Variable | Type | Default | Description |
|---|---|---|---|
| etherpad_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Etherpad docker  project name. |
| etherpad_docker_compose_project_src | derived | `{{ deploy_dir }}` | Etherpad docker  project src. |
| etherpad_docker_compose_services_dir | derived | `{{ services_dir }}` | Etherpad docker  services dir. |
| etherpad_docker_compose_services_state | derived | `{{ compose_service_etherpad_state | default('present') }}` | Etherpad docker  services state. |
| etherpad_docker_environment_default_pad_text | string | `Welcome to OpenTalk` | Etherpad docker environment default pad text. |
| etherpad_docker_environment_suppress_errors_in_pad_text | bool | `True` | Etherpad docker environment suppress errors in pad text. |
| etherpad_docker_environment_trust_proxy | bool | `True` | Etherpad docker environment trust proxy. |
| etherpad_docker_extra_hosts | list | `[]` | Etherpad docker extra hosts. |
| etherpad_docker_image_src | string | `registry.opencode.de/opentalk/etherpad` | Etherpad docker image src. |
| etherpad_docker_labels_traefik_enable | bool | `True` | Etherpad docker labels traefik enable. |
| etherpad_docker_labels_traefik_http_routers_etherpad_rule | derived | `Host(`{{ ot_domain }}`) && PathPrefix(`/etherpad`)` | Etherpad docker labels traefik http routers etherpad rule. |
| etherpad_docker_ports | list | `[]` | Etherpad docker ports. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| config_dir | template | — | Config dir (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| etherpad_version | template | — | Etherpad version (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| to_json | template | — | To json (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.keycloak

| Variable | Type | Default | Description |
|---|---|---|---|
| keycloak_account_theme | string | `opentalk` | Keycloak account theme. |
| keycloak_api_base_url | derived | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) | build_subdomain_url }}/auth/admin/realms/{{ keycloak_realm }}` | Keycloak api base url. |
| keycloak_auth_password | secret[derived] | `{{ keycloak_permanent_admin_password }}` | Keycloak auth password. |
| keycloak_auth_realm | string | `master` | Keycloak auth realm. |
| keycloak_auth_username | derived | `{{ keycloak_permanent_admin_username }}` | Keycloak auth username. |
| keycloak_base_url | derived | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) | build_subdomain_url }}/auth` | Keycloak base url. |
| keycloak_bootstrap_admin_password | secret[derived] | `{{ KC_BOOTSTRAP_ADMIN_PASSWORD }}` | Keycloak bootstrap admin password. |
| keycloak_bootstrap_admin_remove | bool | `True` | Keycloak bootstrap admin remove. |
| keycloak_bootstrap_admin_username | string | `bootstrap_admin` | Keycloak bootstrap admin username. |
| keycloak_db_password | secret[derived] | `{{ KEYCLOAK_DB_PASSWORD }}` | Keycloak db password. |
| keycloak_db_schema | string | `public` | Keycloak db schema. |
| keycloak_db_url_database | string | `keycloak` | Keycloak db url database. |
| keycloak_db_url_host | string | `keycloak-db` | Keycloak db url host. |
| keycloak_db_url_port | int | `5432` | Keycloak db url port. |
| keycloak_db_username | string | `ot` | Keycloak db username. |
| keycloak_db_vendor | string | `postgres` | Keycloak db vendor. |
| keycloak_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Keycloak docker  project name. |
| keycloak_docker_compose_project_src | derived | `{{ deploy_dir }}` | Keycloak docker  project src. |
| keycloak_docker_compose_services_dir | derived | `{{ services_dir }}` | Keycloak docker  services dir. |
| keycloak_docker_compose_services_state | derived | `{{ compose_service_keycloak_state | default('present') }}` | Keycloak docker  services state. |
| keycloak_docker_compose_state | string | `present` | Keycloak docker  state. |
| keycloak_docker_image_src | string | `quay.io/keycloak/keycloak` | Keycloak docker image src. |
| keycloak_health_enabled | string | `true` | Keycloak health enabled. |
| keycloak_health_url | derived | `{{ keycloak_base_url }}/realms/{{ keycloak_auth_realm }}/.well-known/openid-configuration` | Keycloak health url. |
| keycloak_hostname | derived | `{{ (ot_ingress_protocol, keycloak_local_hostname, ot_domain, keycloak_local_port) | build_subdomain_url }}/auth` | Keycloak hostname. |
| keycloak_http_relative_path | string | `/auth` | Keycloak http relative path. |
| keycloak_local_hostname | string | `` | Keycloak local hostname. |
| keycloak_local_port | derived | `{{ ot_ingress_port }}` | Keycloak local port. |
| keycloak_login_theme | string | `opentalk` | Keycloak login theme. |
| keycloak_manage_enabled | bool | `True` | Keycloak manage enabled. |
| keycloak_metrics_enabled | string | `true` | Keycloak metrics enabled. |
| keycloak_permanent_admin_email | derived | `admin@{{ ot_domain | default('localhost') }}` | Keycloak permanent admin email. |
| keycloak_permanent_admin_firstname | string | `Keycloak` | Keycloak permanent admin firstname. |
| keycloak_permanent_admin_lastname | string | `Administrator` | Keycloak permanent admin lastname. |
| keycloak_permanent_admin_password | secret[derived] | `{{ KC_PERMANENT_ADMIN_PASSWORD }}` | Keycloak permanent admin password. |
| keycloak_permanent_admin_username | string | `admin` | Keycloak permanent admin username. |
| keycloak_proxy | string | `edge` | Keycloak proxy. |
| keycloak_proxy_headers | string | `xforwarded` | Keycloak proxy headers. |
| keycloak_realm | string | `opentalk` | Keycloak realm. |
| keycloak_redirectUris_uris | list[str] | {{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/auth/popup_callback<br>{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/auth/callback<br>{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/dashboard<br>{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/dashboard/*<br>{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/room/*<br>{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/join<br>{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/ | Keycloak redirectUris uris. |
| keycloak_spi_sticky_session_encoder_infinispan_should_attach_route | string | `false` | Keycloak spi sticky session encoder infinispan should attach route. |
| keycloak_wait_to_be_reachable_delay | int | `10` | Keycloak wait to be reachable delay. |
| keycloak_wait_to_be_reachable_retries | int | `20` | Keycloak wait to be reachable retries. |
| postInstall_demoUser_email | string | `demo.user@example.com` | PostInstall demoUser email. |
| postInstall_demoUser_enabled | string | `True` | PostInstall demoUser enabled. |
| postInstall_demoUser_firstName | string | `Test` | PostInstall demoUser firstName. |
| postInstall_demoUser_lastName | string | `User` | PostInstall demoUser lastName. |
| postInstall_demoUser_name | string | `demo` | PostInstall demoUser name. |
| postInstall_demoUser_password | secret[derived] | `{{ DEMO_USER_PASSWORD }}` | PostInstall demoUser password. |
| postInstall_demoUser_state | string | `present` | PostInstall demoUser state. |
| postgres_docker_image_src | string | `postgres` | Postgres docker image src. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| config_dir | template | — | Config dir (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| keycloak_version | template | — | Keycloak version (referenced in templates; no default defined). |
| localhost | template | — | Localhost (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| postgres_version | template | — | Postgres version (referenced in templates; no default defined). |
| secret | secret[template] | — | Secret (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.livekit_server

| Variable | Type | Default | Description |
|---|---|---|---|
| livekit_api_key_controller | secret[derived] | `{{ LIVEKIT_API_SECRET }}` | LiveKit api key controller. |
| livekit_docker_compose_project_src | derived | `{{ deploy_dir }}` | LiveKit docker  project src. |
| livekit_docker_image_src | string | `livekit/livekit-server` | LiveKit docker image src. |
| livekit_port | int | `7880` | LiveKit port. |
| livekit_rtc_enable_loopback_candidate | bool | `False` | LiveKit rtc enable loopback candidate. |
| livekit_rtc_ips_excludes | list[str] | 10.0.0.0/8<br>172.16.0.0/12<br>192.168.0.0/16 | LiveKit rtc ips excludes. |
| livekit_rtc_ips_includes | list | `[]` | LiveKit rtc ips includes. |
| livekit_rtc_port_range_end | int | `64535` | LiveKit rtc port range end. |
| livekit_rtc_port_range_start | int | `50000` | LiveKit rtc port range start. |
| livekit_rtc_tcp_port | int | `7881` | LiveKit rtc tcp port. |
| livekit_rtc_use_external_ip | bool | `True` | LiveKit rtc use external ip. |
| livekit_server_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | LiveKit server docker  project name. |
| livekit_server_docker_compose_services_dir | derived | `{{ services_dir }}` | LiveKit server docker  services dir. |
| livekit_server_docker_compose_services_state | derived | `{{ compose_service_livekit_server_state | default('present') }}` | LiveKit server docker  services state. |
| livekit_turn_embedded_domain | derived | `{{ ot_domain }}` | LiveKit turn embedded domain. |
| livekit_turn_embedded_enabled | bool | `True` | LiveKit turn embedded enabled. |
| livekit_turn_embedded_external_tls | bool | `True` | LiveKit turn embedded external tls. |
| livekit_turn_embedded_tls_enabled | bool | `False` | LiveKit turn embedded tls enabled. |
| livekit_turn_embedded_tls_port | int | `5349` | LiveKit turn embedded tls port. |
| livekit_turn_embedded_udp_port | int | `3478` | LiveKit turn embedded udp port. |
| ansible_host | template | — | Ansible host (referenced in templates; no default defined). |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| credential | template | — | Credential (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| host | template | — | Host (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| livekit_logging | template | — | LiveKit logging (referenced in templates; no default defined). |
| livekit_logging_json | template | — | LiveKit logging json (referenced in templates; no default defined). |
| livekit_logging_level | template | — | LiveKit logging level (referenced in templates; no default defined). |
| livekit_prometheus_port | template | — | LiveKit prometheus port (referenced in templates; no default defined). |
| livekit_rtc_node_ip | template | — | LiveKit rtc node ip (referenced in templates; no default defined). |
| livekit_rtc_stun_servers | template | — | LiveKit rtc stun servers (referenced in templates; no default defined). |
| livekit_rtc_turn_servers | template | — | LiveKit rtc turn servers (referenced in templates; no default defined). |
| livekit_rtc_udp_port | template | — | LiveKit rtc udp port (referenced in templates; no default defined). |
| livekit_rtc_use_ice_lite | template | — | LiveKit rtc use ice lite (referenced in templates; no default defined). |
| livekit_version | template | — | LiveKit version (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| port | template | — | Port (referenced in templates; no default defined). |
| protocol | template | — | Protocol (referenced in templates; no default defined). |
| to_json | template | — | To json (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| udp | template | — | Udp (referenced in templates; no default defined). |
| username | template | — | Username (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.mailcrab

| Variable | Type | Default | Description |
|---|---|---|---|
| mailcrab_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Mailcrab docker  project name. |
| mailcrab_docker_compose_project_src | derived | `{{ deploy_dir }}` | Mailcrab docker  project src. |
| mailcrab_docker_compose_services_dir | derived | `{{ services_dir }}` | Mailcrab docker  services dir. |
| mailcrab_docker_compose_services_state | derived | `{{ compose_service_mailcrab_state | default('present') }}` | Mailcrab docker  services state. |
| mailcrab_docker_image_src | string | `marlonb/mailcrab` | Mailcrab docker image src. |
| mailcrab_docker_label_http_routers_minio_console_middlewares | string | `default-auth@file` | Mailcrab docker label http routers minio console middlewares. |
| mailcrab_webui_prefix | string | `/adm/mailcrab` | Mailcrab webui prefix. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| mailcrab_version | template | — | Mailcrab version (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.minio

| Variable | Type | Default | Description |
|---|---|---|---|
| minio_bucket_name | string | `opentalk` | Minio bucket name. |
| minio_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Minio docker  project name. |
| minio_docker_compose_project_src | derived | `{{ deploy_dir }}` | Minio docker  project src. |
| minio_docker_compose_services_dir | derived | `{{ services_dir }}` | Minio docker  services dir. |
| minio_docker_compose_services_state | derived | `{{ compose_service_minio_state | default('present') }}` | Minio docker  services state. |
| minio_docker_image_src | string | `minio/minio` | Minio docker image src. |
| minio_docker_label_http_routers_minio_console_middlewares | string | `minio-stripprefix` | Minio docker label http routers minio console middlewares. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| config_dir | template | — | Config dir (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| http | template | — | Http (referenced in templates; no default defined). |
| https | template | — | Https (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| minio_version | template | — | Minio version (referenced in templates; no default defined). |
| opentalk | template | — | opentalk (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.ndt

| Variable | Type | Default | Description |
|---|---|---|---|
| compose_service_ndt_enabled | derived | `{{ ndt_enable | default(false) }}` | Service ndt enabled. |
| ndt_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Ndt docker  project name. |
| ndt_docker_compose_project_src | derived | `{{ deploy_dir }}` | Ndt docker  project src. |
| ndt_docker_compose_services_dir | derived | `{{ services_dir }}` | Ndt docker  services dir. |
| ndt_docker_compose_services_state | derived | `{{ compose_service_ndt_state | default('present') }}` | Ndt docker  services state. |
| ndt_docker_image_src | string | `measurementlab/ndt-server` | Ndt docker image src. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| ndt_version | template | — | Ndt version (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.opentalk_controller

| Variable | Type | Default | Description |
|---|---|---|---|
| controller_database_host | string | `controller-db` | Controller database host. |
| controller_database_name | string | `controller` | Controller database name. |
| controller_database_password | secret[derived] | `{{ CONTROLLER_DB_PASSWORD }}` | Controller database password. |
| controller_database_port | int | `5432` | Controller database port. |
| controller_database_user | string | `ot` | Controller database user. |
| controller_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Controller docker  project name. |
| controller_docker_compose_project_src | derived | `{{ deploy_dir }}` | Controller docker  project src. |
| controller_docker_compose_services_dir | derived | `{{ services_dir }}` | Controller docker  services dir. |
| controller_docker_compose_services_state | derived | `{{ compose_service_opentalk_controller_state | default('present') }}` | Controller docker  services state. |
| controller_docker_extra_hosts | list[str] | host.docker.internal:host-gateway | Controller docker extra hosts. |
| controller_docker_image_src | string | `registry.opencode.de/opentalk/controller` | Controller docker image src. |
| controller_endpoints_event_invite_external_email_address | string | `true` | Controller endpoints event invite external email address. |
| controller_etherpad_api_key | secret[derived] | `{{ ETHERPAD_API_KEY }}` | Controller etherpad api key. |
| controller_etherpad_url | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/etherpad/` | Controller etherpad url. |
| controller_frontend_base_url | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}` | Controller frontend base url. |
| controller_http_port | int | `11311` | Controller http port. |
| controller_livekit_api_key | string | `controller` | Controller livekit api key. |
| controller_livekit_api_secret | secret[derived] | `{{ LIVEKIT_API_SECRET }}` | Controller livekit api secret. |
| controller_livekit_public_url | derived | `wss://{{ (ot_domain, ot_ingress_port) | build_host }}/livekit` | Controller livekit public url. |
| controller_livekit_service_url | string | `http://host.docker.internal:7880` | Controller livekit service url. |
| controller_logging_default_directives | list[str] | ERROR<br>opentalk=INFO<br>pinky_swear=OFF<br>rustls=WARN<br>mio=ERROR<br>lapin=WARN<br>execution_id=trace | Controller logging default directives. |
| controller_minio_access_key | string | `minioadmin` | Controller minio access key. |
| controller_minio_bucket | string | `opentalk` | Controller minio bucket. |
| controller_minio_secret_key | secret[derived] | `{{ MINIO_ROOT_PASSWORD }}` | Controller minio secret key. |
| controller_minio_uri | string | `http://minio:9000` | Controller minio uri. |
| controller_monitoring_port | int | `11411` | Controller monitoring port. |
| controller_oidc_authority | derived | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` | Controller oidc authority. |
| controller_oidc_controller_client_id | string | `Controller` | Controller oidc controller client id. |
| controller_oidc_controller_client_secret | secret[derived] | `{{ OIDC_CONTROLLER_CLIENT_SECRET }}` | Controller oidc controller client secret. |
| controller_oidc_frontend_client_id | string | `Frontend` | Controller oidc frontend client id. |
| controller_operator_information_data_protection_url | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/data-protection` | Controller operator information data protection url. |
| controller_operator_information_support_email_address | derived | `support@{{ ot_domain }}` | Controller operator information support email address. |
| controller_operator_information_support_phone_number | string | `+49 123 4567890` | Controller operator information support phone number. |
| controller_rabbit_mq_mail_task_queue | string | `opentalk_mailer` | Controller rabbit mq mail task queue. |
| controller_rabbit_mq_max_channels_per_connection | int | `100` | Controller rabbit mq max channels per connection. |
| controller_rabbit_mq_min_connections | int | `10` | Controller rabbit mq min connections. |
| controller_rabbit_mq_recording_task_queue | string | `opentalk_recorder` | Controller rabbit mq recording task queue. |
| controller_rabbit_mq_url | secret[derived] | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` | Controller rabbit mq url. |
| controller_spacedeck_api_key | secret[derived] | `{{ SPACEDECK_API_KEY }}` | Controller spacedeck api key. |
| controller_spacedeck_url | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/whiteboard/` | Controller spacedeck url. |
| controller_user_search_api_base_url | derived | `{{ keycloak_api_base_url }}` | Controller user search api base url. |
| controller_user_search_backend | string | `keycloak_webapi` | Controller user search backend. |
| controller_user_search_users_find_behavior | string | `from_user_search_backend` | Controller user search users find behavior. |
| postgres_docker_image_src | string | `postgres` | Postgres docker image src. |
| Frontend | template | — | Frontend (referenced in templates; no default defined). |
| OPENTALK | template | — | OPENTALK (referenced in templates; no default defined). |
| auth | template | — | Auth (referenced in templates; no default defined). |
| com | template | — | Com (referenced in templates; no default defined). |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| config_dir | template | — | Config dir (referenced in templates; no default defined). |
| controller_authz_synchronize_controllers | template | — | Controller authz synchronize controllers (referenced in templates; no default defined). |
| controller_call_in_default_country_code | template | — | Controller call in default country code (referenced in templates; no default defined). |
| controller_call_in_enable_phone_mapping | template | — | Controller call in enable phone mapping (referenced in templates; no default defined). |
| controller_call_in_tel | template | — | Controller call in tel (referenced in templates; no default defined). |
| controller_database_max_connections | template | — | Controller database max connections (referenced in templates; no default defined). |
| controller_defaults_disabled_features | template | — | Controller defaults disabled features (referenced in templates; no default defined). |
| controller_defaults_screen_share_requires_permission | template | — | Controller defaults screen share requires permission (referenced in templates; no default defined). |
| controller_defaults_user_language | template | — | Controller defaults user language (referenced in templates; no default defined). |
| controller_endpoints_disable_openapi | template | — | Controller endpoints disable openapi (referenced in templates; no default defined). |
| controller_endpoints_disable_users_find | template | — | Controller endpoints disable users find (referenced in templates; no default defined). |
| controller_endpoints_disallow_custom_display_name | template | — | Controller endpoints disallow custom display name (referenced in templates; no default defined). |
| controller_endpoints_users_find_use_kc | template | — | Controller endpoints users find use kc (referenced in templates; no default defined). |
| controller_etcd_urls | template | — | Controller etcd urls (referenced in templates; no default defined). |
| controller_http_cors_allowed_origin | template | — | Controller http cors allowed origin (referenced in templates; no default defined). |
| controller_logging_otlp_tracing_endpoint | template | — | Controller logging otlp tracing endpoint (referenced in templates; no default defined). |
| controller_logging_service_instance_id | template | — | Controller logging service instance id (referenced in templates; no default defined). |
| controller_logging_service_name | template | — | Controller logging service name (referenced in templates; no default defined). |
| controller_logging_service_namespace | template | — | Controller logging service namespace (referenced in templates; no default defined). |
| controller_metrics_allowlist | template | — | Controller metrics allowlist (referenced in templates; no default defined). |
| controller_monitoring_addr | template | — | Controller monitoring addr (referenced in templates; no default defined). |
| controller_oidc_controller_authority | template | — | Controller oidc controller authority (referenced in templates; no default defined). |
| controller_oidc_frontend_authority | template | — | Controller oidc frontend authority (referenced in templates; no default defined). |
| controller_redis_url | template | — | Controller redis url (referenced in templates; no default defined). |
| controller_reports_template_inline | template | — | Controller reports template inline (referenced in templates; no default defined). |
| controller_reports_url | template | — | Controller reports url (referenced in templates; no default defined). |
| controller_shared_folder_directory | template | — | Controller shared folder directory (referenced in templates; no default defined). |
| controller_shared_folder_expiry | template | — | Controller shared folder expiry (referenced in templates; no default defined). |
| controller_shared_folder_password | secret[template] | — | Controller shared folder password (referenced in templates; no default defined). |
| controller_shared_folder_provider | template | — | Controller shared folder provider (referenced in templates; no default defined). |
| controller_shared_folder_url | template | — | Controller shared folder url (referenced in templates; no default defined). |
| controller_shared_folder_username | template | — | Controller shared folder username (referenced in templates; no default defined). |
| controller_tariffs_assignment | template | — | Controller tariffs assignment (referenced in templates; no default defined). |
| controller_tariffs_static_tariff_name | template | — | Controller tariffs static tariff name (referenced in templates; no default defined). |
| controller_tariffs_status_mapping_default | template | — | Controller tariffs status mapping default (referenced in templates; no default defined). |
| controller_tariffs_status_mapping_downgraded | template | — | Controller tariffs status mapping downgraded (referenced in templates; no default defined). |
| controller_tariffs_status_mapping_downgraded_tariff_name | template | — | Controller tariffs status mapping downgraded tariff name (referenced in templates; no default defined). |
| controller_tariffs_status_mapping_paid | template | — | Controller tariffs status mapping paid (referenced in templates; no default defined). |
| controller_tenants_assignment | template | — | Controller tenants assignment (referenced in templates; no default defined). |
| controller_tenants_external_tenant_id_user_attribute_name | template | — | Controller tenants external tenant id user attribute name (referenced in templates; no default defined). |
| controller_tenants_static_tenant_id | template | — | Controller tenants static tenant id (referenced in templates; no default defined). |
| controller_user_search_client_id | template | — | Controller user search client id (referenced in templates; no default defined). |
| controller_user_search_client_secret | secret[template] | — | Controller user search client secret (referenced in templates; no default defined). |
| controller_user_search_external_id_user_attribute_name | template | — | Controller user search external id user attribute name (referenced in templates; no default defined). |
| custom_certificates_ca_root_cert_file | template | — | Custom certificates ca root cert file (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| example | template | — | Example (referenced in templates; no default defined). |
| https | template | — | Https (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| last | template | — | Last (referenced in templates; no default defined). |
| length | template | — | Length (referenced in templates; no default defined). |
| opentalk_controller_version | template | — | Controller version (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| postgres_version | template | — | Postgres version (referenced in templates; no default defined). |
| realms | template | — | Realms (referenced in templates; no default defined). |
| to_json | template | — | To json (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.opentalk_obelisk

| Variable | Type | Default | Description |
|---|---|---|---|
| obelisk_auth_client_id | string | `Obelisk` | Obelisk auth client id. |
| obelisk_auth_client_secret | secret[derived] | `{{ OIDC_OBELISK_CLIENT_SECRET }}` | Obelisk auth client secret. |
| obelisk_auth_issuer | derived | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` | Obelisk auth issuer. |
| obelisk_clock_format | string | `%d.%m.%y %X %Z` | Obelisk clock format. |
| obelisk_controller_domain | derived | `{{ ot_domain }}/controller` | Obelisk controller domain. |
| obelisk_controller_insecure | bool | `False` | Obelisk controller insecure. |
| obelisk_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Obelisk docker  project name. |
| obelisk_docker_compose_project_src | derived | `{{ deploy_dir }}` | Obelisk docker  project src. |
| obelisk_docker_compose_services_dir | derived | `{{ services_dir }}` | Obelisk docker  services dir. |
| obelisk_docker_compose_services_state | derived | `{{ compose_service_opentalk_obelisk_state | default('present') }}` | Obelisk docker  services state. |
| obelisk_docker_extra_hosts | list[str] | host.docker.internal:host-gateway | Obelisk docker extra hosts. |
| obelisk_docker_image_src | string | `registry.opencode.de/opentalk/obelisk` | Obelisk docker image src. |
| obelisk_docker_network_mode | string | `` | Obelisk docker network mode. |
| obelisk_docker_ports | list[str] | 5060:5060 | Obelisk docker ports. |
| obelisk_environment_gst_debug_level | int | `2` | Obelisk environment gst debug level. |
| obelisk_environment_rust_log_level | string | `info` | Obelisk environment rust log level. |
| obelisk_monitoring_addr | string | `0.0.0.0` | Obelisk monitoring addr. |
| obelisk_monitoring_enable | bool | `True` | Obelisk monitoring enable. |
| obelisk_monitoring_port | int | `11411` | Obelisk monitoring port. |
| obelisk_sip_addr | string | `0.0.0.0` | Obelisk sip addr. |
| obelisk_sip_contact | string | `` | Obelisk sip contact. |
| obelisk_sip_enforce_qop | bool | `True` | Obelisk sip enforce qop. |
| obelisk_sip_id | string | `` | Obelisk sip id. |
| obelisk_sip_max_video_bitrate | int | `6000` | Obelisk sip max video bitrate. |
| obelisk_sip_nat_ping_delta | string | `` | Obelisk sip nat ping delta. |
| obelisk_sip_offer_srtp | string | `` | Obelisk sip offer srtp. |
| obelisk_sip_outbound_proxy | string | `` | Obelisk sip outbound proxy. |
| obelisk_sip_password | string | `` | Obelisk sip password. |
| obelisk_sip_port | int | `5060` | Obelisk sip port. |
| obelisk_sip_realm | string | `` | Obelisk sip realm. |
| obelisk_sip_registrar | string | `` | Obelisk sip registrar. |
| obelisk_sip_rtp_port_range_end | int | `49999` | Obelisk sip rtp port range end. |
| obelisk_sip_rtp_port_range_start | int | `40000` | Obelisk sip rtp port range start. |
| obelisk_sip_stun_server | string | `` | Obelisk sip stun server. |
| obelisk_sip_username | string | `` | Obelisk sip username. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| custom_certificates_ca_root_cert_file | template | — | Custom certificates ca root cert file (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| host | template | — | Host (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| length | template | — | Length (referenced in templates; no default defined). |
| obelisk_sip_max_audio_bitrate | template | — | Obelisk sip max audio bitrate (referenced in templates; no default defined). |
| opentalk_obelisk_version | template | — | Obelisk version (referenced in templates; no default defined). |
| to_json | template | — | To json (referenced in templates; no default defined). |

### opentalk_compose.opentalk_outlook_add_in

| Variable | Type | Default | Description |
|---|---|---|---|
| outlook_add_in_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Outlook add in docker  project name. |
| outlook_add_in_docker_compose_project_src | derived | `{{ deploy_dir }}` | Outlook add in docker  project src. |
| outlook_add_in_docker_compose_services_dir | derived | `{{ services_dir }}` | Outlook add in docker  services dir. |
| outlook_add_in_docker_compose_services_state | derived | `{{ compose_service_opentalk_outlook_add_in_state | default('present') }}` | Outlook add in docker  services state. |
| outlook_add_in_docker_image_src | string | `registry.opencode.de/opentalk/opentalk-outlook-add-in` | Outlook add in docker image src. |
| outlook_add_in_labels_traefik_http_middlewares_outlook_add_in_cors_headers_accessControlAllowOriginList | derived | `{{ (ot_ingress_protocol, ot_domain, '*') | build_url }}` | Outlook add in labels traefik http middlewares outlook add in cors headers accessControlAllowOriginList. |
| outlook_add_in_oidc_client_id | string | `OutlookAddin` | Outlook add in oidc client id. |
| outlook_add_in_outlook_url | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/outlook` | Outlook add in outlook url. |
| outlook_add_in_outlook_webapp_url | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}` | Outlook add in outlook webapp url. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| opentalk_outlook_add_in_version | template | — | Outlook add in version (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.opentalk_recorder

| Variable | Type | Default | Description |
|---|---|---|---|
| recorder_auth_client_id | string | `Recorder` | Recorder auth client id. |
| recorder_auth_client_secret | secret[derived] | `{{ OIDC_RECORDER_CLIENT_SECRET }}` | Recorder auth client secret. |
| recorder_auth_issuer | derived | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` | Recorder auth issuer. |
| recorder_controller_domain | string | `controller:11311` | Recorder controller domain. |
| recorder_controller_insecure | string | `true` | Recorder controller insecure. |
| recorder_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Recorder docker  project name. |
| recorder_docker_compose_project_src | derived | `{{ deploy_dir }}` | Recorder docker  project src. |
| recorder_docker_compose_services_dir | derived | `{{ services_dir }}` | Recorder docker  services dir. |
| recorder_docker_compose_services_state | derived | `{{ compose_service_opentalk_recorder_state | default('present') }}` | Recorder docker  services state. |
| recorder_docker_extra_hosts | list[str] | host.docker.internal:host-gateway | Recorder docker extra hosts. |
| recorder_docker_image_src | string | `registry.opencode.de/opentalk/recorder` | Recorder docker image src. |
| recorder_monitoring_port | int | `11411` | Recorder monitoring port. |
| recorder_rabbitmq_queue | string | `opentalk_recorder` | Recorder rabbitmq queue. |
| recorder_rabbitmq_uri | secret[derived] | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` | Recorder rabbitmq uri. |
| recorder_recorder_clock_format | string | `%d.%m.%y %X %Z` | Recorder recorder clock format. |
| recorder_recorder_display | string | `true` | Recorder recorder display. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| custom_certificates_ca_root_cert_file | template | — | Custom certificates ca root cert file (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| intel | template | — | Intel (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| length | template | — | Length (referenced in templates; no default defined). |
| opentalk_recorder_version | template | — | Recorder version (referenced in templates; no default defined). |
| recorder_hardware_acceleration_device | template | — | Recorder hardware acceleration device (referenced in templates; no default defined). |
| recorder_hardware_acceleration_manufacturer | template | — | Recorder hardware acceleration manufacturer (referenced in templates; no default defined). |

### opentalk_compose.opentalk_smtp_mailer

| Variable | Type | Default | Description |
|---|---|---|---|
| mailer_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Mailer docker  project name. |
| mailer_docker_compose_project_src | derived | `{{ deploy_dir }}` | Mailer docker  project src. |
| mailer_docker_compose_services_dir | derived | `{{ services_dir }}` | Mailer docker  services dir. |
| mailer_docker_compose_services_state | derived | `{{ compose_service_opentalk_smtp_mailer_state | default('present') }}` | Mailer docker  services state. |
| mailer_docker_image_src | string | `registry.opencode.de/opentalk/smtp-mailer` | Mailer docker image src. |
| mailer_frontend_base_url | derived | `https://{{ ot_domain }}` | Mailer frontend base url. |
| mailer_languages_default_language | string | `de-DE` | Mailer languages default language. |
| mailer_monitoring_addr | string | `0.0.0.0` | Mailer monitoring addr. |
| mailer_monitoring_port | int | `11411` | Mailer monitoring port. |
| mailer_rabbit_mq_mail_task_queue | string | `opentalk_mailer` | Mailer rabbit mq mail task queue. |
| mailer_rabbit_mq_url | secret[derived] | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` | Mailer rabbit mq url. |
| mailer_smtp_from_email | derived | `no-reply@{{ ot_domain }}` | Mailer smtp from email. |
| mailer_smtp_from_name | derived | `OpenTalk - {{ ot_domain }}` | Mailer smtp from name. |
| mailer_smtp_smtp_server | string | `smtp://mailcrab:1025?disable_starttls=true` | Mailer smtp smtp server. |
| mailer_support_contact_mail | derived | `support@{{ ot_domain }}` | Mailer support contact mail. |
| mailer_support_contact_phone | string | `+49 123 456789` | Mailer support contact phone. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| mailer_frontend_data_protection_url | template | — | Mailer frontend data protection url (referenced in templates; no default defined). |
| mailer_template_builder_dashboard_event_link_builder | template | — | Mailer template builder dashboard event link builder (referenced in templates; no default defined). |
| mailer_template_builder_guest_link_builder | template | — | Mailer template builder guest link builder (referenced in templates; no default defined). |
| mailer_template_builder_join_link_builder | template | — | Mailer template builder join link builder (referenced in templates; no default defined). |
| mailer_templates_external_event_cancellation | template | — | Mailer templates external event cancellation (referenced in templates; no default defined). |
| mailer_templates_external_event_update | template | — | Mailer templates external event update (referenced in templates; no default defined). |
| mailer_templates_external_invite | template | — | Mailer templates external invite (referenced in templates; no default defined). |
| mailer_templates_external_uninvite | template | — | Mailer templates external uninvite (referenced in templates; no default defined). |
| mailer_templates_registered_event_cancellation | template | — | Mailer templates registered event cancellation (referenced in templates; no default defined). |
| mailer_templates_registered_event_update | template | — | Mailer templates registered event update (referenced in templates; no default defined). |
| mailer_templates_registered_invite | template | — | Mailer templates registered invite (referenced in templates; no default defined). |
| mailer_templates_registered_uninvite | template | — | Mailer templates registered uninvite (referenced in templates; no default defined). |
| mailer_templates_unregistered_event_cancellation | template | — | Mailer templates unregistered event cancellation (referenced in templates; no default defined). |
| mailer_templates_unregistered_event_update | template | — | Mailer templates unregistered event update (referenced in templates; no default defined). |
| mailer_templates_unregistered_invite | template | — | Mailer templates unregistered invite (referenced in templates; no default defined). |
| mailer_templates_unregistered_uninvite | template | — | Mailer templates unregistered uninvite (referenced in templates; no default defined). |
| opentalk_smtp_mailer_version | template | — | Smtp mailer version (referenced in templates; no default defined). |

### opentalk_compose.opentalk_web_frontend

| Variable | Type | Default | Description |
|---|---|---|---|
| opentalk_web_frontend_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Web frontend docker  project name. |
| opentalk_web_frontend_docker_compose_project_src | derived | `{{ deploy_dir }}` | Web frontend docker  project src. |
| opentalk_web_frontend_docker_compose_services_dir | derived | `{{ services_dir }}` | Web frontend docker  services dir. |
| opentalk_web_frontend_docker_compose_services_state | derived | `{{ compose_service_opentalk_web_frontend_state | default('present') }}` | Web frontend docker  services state. |
| opentalk_web_frontend_docker_extra_hosts | list | `[]` | Web frontend docker extra hosts. |
| opentalk_web_frontend_docker_image_src | string | `registry.opencode.de/opentalk/web-frontend` | Web frontend docker image src. |
| opentalk_web_frontend_environment_base_url | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}` | Web frontend environment base url. |
| opentalk_web_frontend_environment_change_password_url | derived | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}/account/` | Web frontend environment change password url. |
| opentalk_web_frontend_environment_controller_host | derived | `{{ (ot_domain, ot_ingress_port) | build_host }}/controller` | Web frontend environment controller host. |
| opentalk_web_frontend_environment_error_report_address | derived | `{{ ERROR_REPORT_ADDRESS | default('reports@example.com') }}` | Web frontend environment error report address. |
| opentalk_web_frontend_environment_feature_protocol | derived | `{{ FEATURE_PROTOCOL | default(true) }}` | Web frontend environment feature protocol. |
| opentalk_web_frontend_environment_feature_recording | derived | `{{ FEATURE_RECORDING | default(false) }}` | Web frontend environment feature recording. |
| opentalk_web_frontend_environment_feature_timer | derived | `{{ FEATURE_TIMER | default(true) }}` | Web frontend environment feature timer. |
| opentalk_web_frontend_environment_feature_user_search | derived | `{{ FEATURE_USER_SEARCH | default(true) }}` | Web frontend environment feature user search. |
| opentalk_web_frontend_environment_feature_whiteboard | derived | `{{ FEATURE_WHITEBOARD | default(true) }}` | Web frontend environment feature whiteboard. |
| opentalk_web_frontend_environment_insecure | bool | `False` | Web frontend environment insecure. |
| opentalk_web_frontend_environment_is_beta_release | derived | `{{ IS_BETA_RELEASE | default(false) }}` | Web frontend environment is beta release. |
| opentalk_web_frontend_environment_libravatar_default_image | derived | `{{ LIBRAVATAR_DEFAULT_IMAGE | default('identicon') }}` | Web frontend environment libravatar default image. |
| opentalk_web_frontend_environment_livekit_server_url | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}/livekit` | Web frontend environment livekit server url. |
| opentalk_web_frontend_environment_ndt_server | derived | `{{ (ot_domain, ot_ingress_port) | build_host }}` | Web frontend environment ndt server. |
| opentalk_web_frontend_environment_oidc_client_id | derived | `{{ controller_oidc_frontend_client_id | default('Frontend') }}` | Web frontend environment oidc client id. |
| opentalk_web_frontend_environment_oidc_issuer | derived | `{{ keycloak_base_url }}/realms/{{ keycloak_realm }}` | Web frontend environment oidc issuer. |
| opentalk_web_frontend_environment_suppress_browser_compatibility_info | bool | `False` | Web frontend environment suppress browser compatibility info. |
| opentalk_web_frontend_environment_video_backgrounds | string | `[{ …` | Web frontend environment video backgrounds. |
| opentalk_web_frontend_environment_waiting_room_default_value | derived | `{{ WAITING_ROOM_DEFAULT_VALUE | default(false) }}` | Web frontend environment waiting room default value. |
| opentalk_web_frontend_labels_traefik_enable | bool | `True` | Web frontend labels traefik enable. |
| opentalk_web_frontend_labels_traefik_http_middlewares_webapp_cors_headers_accessControlAllowOriginList | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}` | Web frontend labels traefik http middlewares webapp cors headers accessControlAllowOriginList. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| domain | template | — | Domain (referenced in templates; no default defined). |
| endmacro | template | — | Endmacro (referenced in templates; no default defined). |
| host | template | — | Host (referenced in templates; no default defined). |
| indent | template | — | Indent (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| macro | template | — | Macro (referenced in templates; no default defined). |
| opentalk_product_version | template | — | Product version (referenced in templates; no default defined). |
| opentalk_web_frontend_version | template | — | Web frontend version (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| port | template | — | Port (referenced in templates; no default defined). |
| protocol | template | — | Protocol (referenced in templates; no default defined). |
| string | template | — | String (referenced in templates; no default defined). |
| to_json | template | — | To json (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| url | template | — | Url (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.rabbitmq

| Variable | Type | Default | Description |
|---|---|---|---|
| rabbitmq_default_pass | secret[derived] | `{{ RABBITMQ_PASSWORD }}` | RabbitMQ default pass. |
| rabbitmq_default_user | string | `ot` | RabbitMQ default user. |
| rabbitmq_default_vhost | string | `/` | RabbitMQ default vhost. |
| rabbitmq_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | RabbitMQ docker  project name. |
| rabbitmq_docker_compose_project_src | derived | `{{ deploy_dir }}` | RabbitMQ docker  project src. |
| rabbitmq_docker_compose_services_dir | derived | `{{ services_dir }}` | RabbitMQ docker  services dir. |
| rabbitmq_docker_compose_services_state | derived | `{{ compose_service_rabbitmq_state | default('present') }}` | RabbitMQ docker  services state. |
| rabbitmq_docker_image_src | string | `rabbitmq` | RabbitMQ docker image src. |
| rabbitmq_docker_ports | list | `[]` | RabbitMQ docker ports. |
| rabbitmq_management_path_prefix | string | `/adm/rabbitmq` | RabbitMQ management path prefix. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| config_dir | template | — | Config dir (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| rabbitmq_version | template | — | RabbitMQ version (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.secrets

| Variable | Type | Default | Description |
|---|---|---|---|
| CONTROLLER_DB_PASSWORD | secret[template] | — | CONTROLLER DB PASSWORD (referenced in templates; no default defined). |
| DEMO_USER_PASSWORD | secret[template] | — | DEMO USER PASSWORD (referenced in templates; no default defined). |
| ETHERPAD_API_KEY | secret[template] | — | ETHERPAD API KEY (referenced in templates; no default defined). |
| KC_BOOTSTRAP_ADMIN_PASSWORD | secret[template] | — | KC BOOTSTRAP ADMIN PASSWORD (referenced in templates; no default defined). |
| KC_PERMANENT_ADMIN_PASSWORD | secret[template] | — | KC PERMANENT ADMIN PASSWORD (referenced in templates; no default defined). |
| KEYCLOAK_DB_PASSWORD | secret[template] | — | KEYCLOAK DB PASSWORD (referenced in templates; no default defined). |
| LIVEKIT_API_SECRET | secret[template] | — | LIVEKIT API SECRET (referenced in templates; no default defined). |
| MINIO_ACC_KEY | secret[template] | — | MINIO ACC KEY (referenced in templates; no default defined). |
| MINIO_ROOT_PASSWORD | secret[template] | — | MINIO ROOT PASSWORD (referenced in templates; no default defined). |
| MINIO_SECRET_KEY | secret[template] | — | MINIO SECRET KEY (referenced in templates; no default defined). |
| OIDC_CONTROLLER_CLIENT_SECRET | secret[template] | — | OIDC CONTROLLER CLIENT SECRET (referenced in templates; no default defined). |
| OIDC_FRONTEND_CLIENT_SECRET | secret[template] | — | OIDC FRONTEND CLIENT SECRET (referenced in templates; no default defined). |
| OIDC_OBELISK_CLIENT_SECRET | secret[template] | — | OIDC OBELISK CLIENT SECRET (referenced in templates; no default defined). |
| OIDC_RECORDER_CLIENT_SECRET | secret[template] | — | OIDC RECORDER CLIENT SECRET (referenced in templates; no default defined). |
| RABBITMQ_ERLANG_COOKIE | template | — | RABBITMQ ERLANG COOKIE (referenced in templates; no default defined). |
| RABBITMQ_PASSWORD | secret[template] | — | RABBITMQ PASSWORD (referenced in templates; no default defined). |
| SPACEDECK_API_KEY | secret[template] | — | SPACEDECK API KEY (referenced in templates; no default defined). |
| SPACEDECK_INVITE_CODE | template | — | SPACEDECK INVITE CODE (referenced in templates; no default defined). |
| TRAEFIK_DASHBOARD_ADMIN_PASSWORD | secret[template] | — | TRAEFIK DASHBOARD ADMIN PASSWORD (referenced in templates; no default defined). |
| TRAEFIK_DASHBOARD_ADMIN_PASSWORD_SALT | template | — | TRAEFIK DASHBOARD ADMIN PASSWORD SALT (referenced in templates; no default defined). |
| secret | secret[template] | — | Secret (referenced in templates; no default defined). |

### opentalk_compose.self_signed_cert

| Variable | Type | Default | Description |
|---|---|---|---|
| custom_certificates_certs_dir | string | `/etc/opentalk-compose/ssl` | Custom certificates certs dir. |
| self_signed_certificates_ca_certificate_validity_days | int | `365` | Self signed certificates ca certificate validity days. |
| self_signed_certificates_ca_common_name | derived | `{{ ot_domain }}` | Self signed certificates ca common name. |
| self_signed_certificates_ca_country | string | `DE` | Self signed certificates ca country. |
| self_signed_certificates_ca_dir | derived | `{{ custom_certificates_certs_dir }}/ca` | Self signed certificates ca dir. |
| self_signed_certificates_ca_email_address | derived | `admin@{{ ot_domain }}` | Self signed certificates ca email address. |
| self_signed_certificates_ca_key_size | int | `4096` | Self signed certificates ca key size. |
| self_signed_certificates_ca_locality | string | `Berlin` | Self signed certificates ca locality. |
| self_signed_certificates_ca_organization | string | `my CA` | Self signed certificates ca organization. |
| self_signed_certificates_ca_organizational_unit | string | `CA` | Self signed certificates ca organizational unit. |
| self_signed_certificates_ca_state | string | `Berlin` | Self signed certificates ca state. |
| self_signed_certificates_generate | bool | `False` | Self signed certificates generate. |
| self_signed_certificates_install_system_trust | bool | `True` | Self signed certificates install system trust. |
| self_signed_certificates_server_certificate_validity_days | int | `365` | Self signed certificates server certificate validity days. |
| self_signed_certificates_server_common_name | derived | `{{ ot_domain }}` | Self signed certificates server common name. |
| self_signed_certificates_server_country | string | `DE` | Self signed certificates server country. |
| self_signed_certificates_server_email_address | derived | `admin@{{ ot_domain }}` | Self signed certificates server email address. |
| self_signed_certificates_server_key_size | int | `4096` | Self signed certificates server key size. |
| self_signed_certificates_server_locality | string | `Berlin` | Self signed certificates server locality. |
| self_signed_certificates_server_organization | string | `my Company` | Self signed certificates server organization. |
| self_signed_certificates_server_organizational_unit | string | `IT Security` | Self signed certificates server organizational unit. |
| self_signed_certificates_server_san_dns | list[str] | {{ ot_domain }}<br>*.{{ ot_domain }} | Self signed certificates server san dns. |
| self_signed_certificates_server_san_ip | list[str] | 127.0.0.1<br>::1 | Self signed certificates server san ip. |
| self_signed_certificates_server_state | string | `Berlin` | Self signed certificates server state. |
| self_signed_certificates_ssl_certificate_ca_file | derived | `{{ custom_certificates_ca_root_cert_file | default('ca.crt') }}` | Self signed certificates ssl certificate ca file. |
| self_signed_certificates_ssl_certificate_fullchain_file | derived | `{{ custom_certificates_ssl_certificate_fullchain_file | default('fullchain.crt') }}` | Self signed certificates ssl certificate fullchain file. |
| self_signed_certificates_ssl_certificate_key_file | derived | `{{ custom_certificates_ssl_certificate_key_file | default('privkey.key') }}` | Self signed certificates ssl certificate key file. |
| self_signed_certificates_system_trust_dest_override | string | `` | Self signed certificates system trust dest override. |
| self_signed_certificates_system_trust_name | string | `opentalk-selfsigned-ca.crt` | Self signed certificates system trust name. |

### opentalk_compose.spacedeck

| Variable | Type | Default | Description |
|---|---|---|---|
| spacedeck_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Spacedeck docker  project name. |
| spacedeck_docker_compose_project_src | derived | `{{ deploy_dir }}` | Spacedeck docker  project src. |
| spacedeck_docker_compose_services_dir | derived | `{{ services_dir }}` | Spacedeck docker  services dir. |
| spacedeck_docker_compose_services_state | derived | `{{ compose_service_spacedeck_state | default('present') }}` | Spacedeck docker  services state. |
| spacedeck_docker_environment_prefix | string | `/whiteboard` | Spacedeck docker environment prefix. |
| spacedeck_docker_environment_sd_api_token | string | `/run/secrets/spacedeck_api_secret` | Spacedeck docker environment sd api token. |
| spacedeck_docker_environment_sd_endpoint | derived | `{{ (ot_ingress_protocol, ot_domain, ot_ingress_port) | build_url }}` | Spacedeck docker environment sd endpoint. |
| spacedeck_docker_environment_sd_host | string | `0.0.0.0` | Spacedeck docker environment sd host. |
| spacedeck_docker_environment_sd_invite_code | string | `/run/secrets/spacedeck_invite_secret` | Spacedeck docker environment sd invite code. |
| spacedeck_docker_environment_sd_port | int | `9666` | Spacedeck docker environment sd port. |
| spacedeck_docker_extra_hosts | list | `[]` | Spacedeck docker extra hosts. |
| spacedeck_docker_image_src | string | `registry.opencode.de/opentalk/spacedeck` | Spacedeck docker image src. |
| spacedeck_docker_labels_traefik_enable | bool | `True` | Spacedeck docker labels traefik enable. |
| spacedeck_docker_ports | list | `[]` | Spacedeck docker ports. |
| compose_project_name | template | — | Project name (referenced in templates; no default defined). |
| config_dir | template | — | Config dir (referenced in templates; no default defined). |
| custom_certificates_ca_root_cert_file | template | — | Custom certificates ca root cert file (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| length | template | — | Length (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| spacedeck_version | template | — | Spacedeck version (referenced in templates; no default defined). |
| to_json | template | — | To json (referenced in templates; no default defined). |
| traefik_letsencrypt_enable | template | — | Traefik letsencrypt enable (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

### opentalk_compose.traefik

| Variable | Type | Default | Description |
|---|---|---|---|
| traefik_accessLog_bufferingSize | int | `100` | Traefik accessLog bufferingSize. |
| traefik_api_dashboard | bool | `True` | Traefik api dashboard. |
| traefik_api_insecure | bool | `False` | Traefik api insecure. |
| traefik_basicAuth_dashboard_admin | string | `admin` | Traefik basicAuth dashboard admin. |
| traefik_basicAuth_dashboard_admin_password | secret[derived] | `{{ TRAEFIK_DASHBOARD_ADMIN_PASSWORD }}` | Traefik basicAuth dashboard admin password. |
| traefik_basicAuth_dashboard_admin_password_salt | derived | `{{ TRAEFIK_DASHBOARD_ADMIN_PASSWORD_SALT }}` | Traefik basicAuth dashboard admin password salt. |
| traefik_docker_compose_project_name | derived | `{{ compose_project_name | default('opentalk') }}` | Traefik docker  project name. |
| traefik_docker_compose_project_src | derived | `{{ deploy_dir }}` | Traefik docker  project src. |
| traefik_docker_compose_services_dir | derived | `{{ services_dir }}` | Traefik docker  services dir. |
| traefik_docker_compose_services_state | derived | `{{ compose_service_traefik_state | default('present') }}` | Traefik docker  services state. |
| traefik_docker_extra_hosts | list[str] | host.docker.internal:host-gateway | Traefik docker extra hosts. |
| traefik_docker_image_src | string | `traefik` | Traefik docker image src. |
| traefik_docker_label_http_routers_dashboard_middlewares | string | `dashboard-strip, default-auth@file` | Traefik docker label http routers dashboard middlewares. |
| traefik_docker_ports | list | {'target': 80, 'published': 80, 'mode': 'host'}<br>{'target': 443, 'published': 443, 'mode': 'host'}<br>{'target': 8080, 'published': 7777} | Traefik docker ports. |
| traefik_docker_restart | string | `always` | Traefik docker restart. |
| traefik_entryPoints_ping_address | string | `:8082` | Traefik entryPoints ping address. |
| traefik_entryPoints_web_address | string | `:80` | Traefik entryPoints web address. |
| traefik_entryPoints_websecure_address | string | `:443` | Traefik entryPoints websecure address. |
| traefik_entryPoints_websecure_http_tls | dict | `{}` | Traefik entryPoints websecure http tls. |
| traefik_global_checkNewVersion | bool | `True` | Traefik global checkNewVersion. |
| traefik_global_sendAnonymousUsage | bool | `False` | Traefik global sendAnonymousUsage. |
| traefik_letsencrypt_acme_caServer | string | `https://acme-v02.api.letsencrypt.org/directory` | Traefik letsencrypt acme caServer. |
| traefik_letsencrypt_acme_email | derived | `ops@{{ ot_domain }}` | Traefik letsencrypt acme email. |
| traefik_letsencrypt_acme_httpChallenge_entryPoint | string | `web` | Traefik letsencrypt acme httpChallenge entryPoint. |
| traefik_letsencrypt_acme_storage | string | `/data/acme.json` | Traefik letsencrypt acme storage. |
| traefik_letsencrypt_enable | bool | `True` | Traefik letsencrypt enable. |
| traefik_log_level | string | `INFO` | Traefik log level. |
| traefik_ping_entryPoint | string | `ping` | Traefik ping entryPoint. |
| traefik_providers_docker_exposedByDefault | bool | `False` | Traefik providers docker exposedByDefault. |
| traefik_providers_file_directory | string | `/etc/traefik/dynamic` | Traefik providers file directory. |
| traefik_providers_file_watch | bool | `True` | Traefik providers file watch. |
| bcrypt | template | — | Bcrypt (referenced in templates; no default defined). |
| csp_allowed_extra_connect_src | template | — | Csp allowed extra connect src (referenced in templates; no default defined). |
| custom_certificates_enabled | template | — | Custom certificates enabled (referenced in templates; no default defined). |
| custom_certificates_fullchain_cert_file | template | — | Custom certificates fullchain cert file (referenced in templates; no default defined). |
| custom_certificates_private_key_file | template | — | Custom certificates private key file (referenced in templates; no default defined). |
| default | template | — | Default (referenced in templates; no default defined). |
| defined | template | — | Defined (referenced in templates; no default defined). |
| docker_network_name | template | — | Docker network name (referenced in templates; no default defined). |
| hashtype | template | — | Hashtype (referenced in templates; no default defined). |
| is | template | — | Is (referenced in templates; no default defined). |
| livekit_turn_embedded_enabled | template | — | LiveKit turn embedded enabled (referenced in templates; no default defined). |
| mode | template | — | Mode (referenced in templates; no default defined). |
| ot_domain | template | — | Ot domain (referenced in templates; no default defined). |
| password_hash | template | — | Password hash (referenced in templates; no default defined). |
| published | template | — | Published (referenced in templates; no default defined). |
| salt | template | — | Salt (referenced in templates; no default defined). |
| target | template | — | Target (referenced in templates; no default defined). |
| to_json | template | — | To json (referenced in templates; no default defined). |
| traefik_version | template | — | Traefik version (referenced in templates; no default defined). |
| web | template | — | Web (referenced in templates; no default defined). |
| websecure | template | — | Websecure (referenced in templates; no default defined). |

---
Generated on: 2025-11-18
