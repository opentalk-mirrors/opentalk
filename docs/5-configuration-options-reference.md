# OpenTalk configuration options reference

Notes:

- Dot notation denotes nesting (section.subkey.subkey).
- Defaults show literal values from defaults; Jinja placeholders like {{ VAR }} are filled from settings/secrets at render time.

## Deployment

| Parameter | Type | Default | Description |
|---|---|---|---|
| Deployment.environment | string | `prod` | Environment label. |
| Deployment.force_compose_override | bool | `true` | Always re-generate compose overrides. |
| Deployment.force_settings_yaml | bool | `false` | Require settings.yml when needed. |
| Deployment.scenario[] | list[string] | `external_keycloak`<br>`external_livekit`<br>`external_recorder` | Scenario toggles for using external components. |

## Custom certificates and CA

| Parameter | Type | Default | Description |
|---|---|---|---|
| CustomCertificates.enabled | bool | `false` | Use custom TLS certs. |
| CustomCertificates.ssl_certificate_fullchain_file | string | `server_fullchain.crt` | Fullchain PEM file. |
| CustomCertificates.ssl_certificate_key_file | string | `server.key` | Private key PEM file. |
| CustomCA.enabled | bool | `false` | Trust an additional CA. |
| CustomCA.ssl_ca_file | string | `ca.crt` | Custom CA certificate. |

## Post‑install demo user

| Parameter | Type | Default | Description |
|---|---|---|---|
| postInstall.demoUser.state | string | `present` | Ensure user present/absent. |
| postInstall.demoUser.enabled | bool | `True` | Toggle demo user. |
| postInstall.demoUser.name | string | `demo` | Username. |
| postInstall.demoUser.password | secret[string] | `{{ DEMO_USER_PASSWORD }}` | Password. |
| postInstall.demoUser.email | string | `demo.user@{{ OT_DOMAIN }}` | Email. |
| postInstall.demoUser.firstName | string | `Test` | First name. |
| postInstall.demoUser.lastName | string | `User` | Last name. |

## Ofelia (backup/housekeeping)

| Parameter | Type | Default | Description |
|---|---|---|---|
| Ofelia.jobs.createBackup.name | string | `full_backup` | Job name. |
| Ofelia.jobs.createBackup.type | string | `run` | Job type. |
| Ofelia.jobs.createBackup.network | string | `opentalk_default` | Docker network. |
| Ofelia.jobs.createBackup.schedule | string | `@daily` | Schedule. |
| Ofelia.jobs.createBackup.image | string | `{{ CFORGE_IMAGE_SRC }}:{{ CFORGE_VERSION }}` | Job image. |
| Ofelia.jobs.createBackup.command | string | `ansible-playbook ... -t createBackup` | Backup command. |
| Ofelia.jobs.createBackup.volumes | list[string] | `/opt/.../ansible:/ansible`<br>`/opt/.../config:/config`<br>`/var/opt/.../data:/data`<br>`/var/opt/.../backups:/backups` | Required mounts. |
| Ofelia.jobs.createBackup.environment | list[string] | `ENABLE_CFORGE=true` | Extra env. |
| Ofelia.jobs.cleanBackup.name | string | `backup_retention` | Job name. |
| Ofelia.jobs.cleanBackup.type | string | `run` | Job type. |
| Ofelia.jobs.cleanBackup.schedule | string | `@daily` | Schedule. |
| Ofelia.jobs.cleanBackup.image | string | `{{ CFORGE_IMAGE_SRC }}:{{ CFORGE_VERSION }}` | Job image. |
| Ofelia.jobs.cleanBackup.command | string | `find /backups -maxdepth 1 -mtime +6 -type f -delete` | Retention (6 days). |
| Ofelia.jobs.cleanBackup.volumes | list[string] | `/var/opt/opentalk-compose/backups:/backups` | Backups mount. |
| Ofelia.global.smtpHost | string | `""` | SMTP host for reports. |
| Ofelia.global.smtpPort | int | `""` | SMTP port. |
| Ofelia.global.smtpUser | string | `""` | SMTP user. |
| Ofelia.global.smtpPassword | secret[string] | `""` | SMTP password. |
| Ofelia.global.smtpTlsSkipVerify | bool | `true` | Skip TLS verify. |
| Ofelia.global.emailTo | string | `admin@{{ OT_DOMAIN }}` | Report recipient. |
| Ofelia.global.emailFrom | string | `no-reply@{{ OT_DOMAIN }}` | Report sender. |
| Ofelia.global.mailOnlyOnError | bool | `false` | Mail only on error. |
| Ofelia.global.saveFolder | string | — | Directory to write reports. |
| Ofelia.global.saveOnlyOnError | bool | — | Save report files only on error. |

## Keycloak

| Parameter | Type | Default | Description |
|---|---|---|---|
| Keycloak.mode | enum | `external` | `external` or bundled `local`. |
| Keycloak.manage_enabled | bool | `true` | Auto‑manage realm/clients. |
| Keycloak.auth.username | string | `admin` | Bootstrap admin user. |
| Keycloak.auth.password | secret[string] | `{{ KC_BOOTSTRAP_ADMIN_PASSWORD }}` | Bootstrap admin password. |
| Keycloak.auth.realm | string | `master` | Admin realm. |
| Keycloak.realm | string | `opentalk` | App realm. |
| Keycloak.login_theme | string | `opentalk` | Login theme. |
| Keycloak.account_theme | string | `opentalk` | Account theme. |
| Keycloak.base_url | url | `https://{{ OT_DOMAIN }}/auth` | Public base URL. |
| Keycloak.api_base_url | url | `https://{{ OT_DOMAIN }}/auth/admin/realms/opentalk` | Admin API base. |
| Keycloak.hostname | string | `{{ OT_DOMAIN | default('keycloak') }}` | Advertised hostname. |
| Keycloak.proxy | enum | `edge` | Proxy mode. |
| Keycloak.proxy_headers | enum | `xforwarded` | Trusted headers. |
| Keycloak.db.vendor | enum | `postgres` | DB vendor. |
| Keycloak.db.username | string | `ot` | DB user. |
| Keycloak.db.password | secret[string] | `{{ KEYCLOAK_DB_PASSWORD }}` | DB password. |
| Keycloak.db.schema | string | `public` | DB schema. |
| Keycloak.db.url.database | string | `keycloak` | DB name. |
| Keycloak.db.url.host | string | `keycloak-db` | DB host. |
| Keycloak.db.url.port | int | `5432` | DB port. |
| Keycloak.health_enabled | bool | `"true"` | Health endpoints. |
| Keycloak.metrics_enabled | bool | `"true"` | Metrics endpoints. |
| Keycloak.health_url | url | `https://{{ OT_DOMAIN }}/auth/realms/master` | Health URL. |
| Keycloak.http.relative_path | string | `/auth` | Context path. |
| Keycloak.spi_sticky_session_encoder_infinispan_should_attach_route | bool | `"false"` | Sticky route cookie. |
| Keycloak.redirectUris.uris | list[url] | `https://{{ OT_DOMAIN }}/auth/popup_callback`<br>`https://{{ OT_DOMAIN }}/auth/callback`<br>`https://{{ OT_DOMAIN }}/dashboard`<br>`https://{{ OT_DOMAIN }}/dashboard/*`<br>`https://{{ OT_DOMAIN }}/room/*`<br>`https://{{ OT_DOMAIN }}/join`<br>`https://{{ OT_DOMAIN }}/` | Allowed redirect URIs. |

## Controller

| Parameter | Type | Default | Description |
|---|---|---|---|
| Controller.frontend.base_url | url | `https://{{ OT_DOMAIN }}` | Public base URL. |
| Controller.operator_information.data_protection_url | url | `https://{{ OT_DOMAIN }}/data-protection` | Privacy link. |
| Controller.operator_information.support_phone_number | string | `+49 123 4567890` | Support phone. |
| Controller.operator_information.support_email_address | string | `support@{{ OT_DOMAIN }}` | Support email. |
| Controller.logging.default_directives | list[string] | `ERROR`<br>`opentalk=INFO`<br>`pinky_swear=OFF`<br>`rustls=WARN`<br>`mio=ERROR`<br>`lapin=WARN`<br>`execution_id=trace` | Log directives. |
| Controller.logging.otlp_tracing_endpoint | url | `http://localhost:4317` | OTLP exporter. |
| Controller.logging.service_name | string | `controller` | Tracing: service.name. |
| Controller.logging.service_namespace | string | `opentalk` | Tracing: namespace. |
| Controller.logging.service_instance_id | string | `627cc493-f...` | Tracing: instance id. |
| Controller.database.name | string | `controller` | DB name. |
| Controller.database.user | string | `ot` | DB user. |
| Controller.database.password | secret[string] | `{{ CONTROLLER_DB_PASSWORD }}` | DB password. |
| Controller.database.host | string | `controller-db` | DB host. |
| Controller.database.port | int | `5432` | DB port. |
| Controller.database.max_connections | int | `100` | Max connections. |
| Controller.monitoring.addr | ip | `0.0.0.0` | Bind address. |
| Controller.monitoring.port | int | `11411` | Metrics port. |
| Controller.http.port | int | `11311` | HTTP listen (internal). |
| Controller.oidc.authority | url | `{{ Keycloak.base_url }}/realms/{{ Keycloak.realm }}` | Issuer. |
| Controller.oidc.frontend.authority | url | same as above | Override issuer for frontend. |
| Controller.oidc.frontend.client_id | string | `Webapp` | Frontend client. |
| Controller.oidc.controller.authority | url | same as above | Override issuer for backend. |
| Controller.oidc.controller.client_id | string | `OIDC` | Backend client id. |
| Controller.oidc.controller.client_secret | secret[string] | `{{ OIDC_CONTROLLER_CLIENT_SECRET }}` | Backend client secret. |
| Controller.user_search.backend | enum | `keycloak_webapi` | User search backend. |
| Controller.user_search.api_base_url | url | `{{ Keycloak.api_base_url }}` | Admin API base. |
| Controller.user_search.users_find_behavior | enum | `from_user_search_backend` | Find behavior. |
| Controller.user_search.client_id | string | `Controller` | Client id for search. |
| Controller.user_search.client_secret | secret[string] | `{{ OIDC_CONTROLLER_CLIENT_SECRET }}` | Client secret for search. |
| Controller.livekit.public_url | url | `wss://{{ OT_DOMAIN }}/livekit` | Public WS URL. |
| Controller.livekit.service_url | url | `http://host.docker.internal:7880` | Internal LK URL. |
| Controller.livekit.api_key | string | `controller` | LiveKit API key name. |
| Controller.livekit.api_secret | secret[string] | `{{ LIVEKIT_API_SECRET }}` | LiveKit API secret. |
| Controller.rabbit_mq.url | url | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` | AMQP URL. |
| Controller.rabbit_mq.mail_task_queue | string | `opentalk_mailer` | Mail queue. |
| Controller.rabbit_mq.recording_task_queue | string | `opentalk_recorder` | Recorder queue. |
| Controller.rabbit_mq.min_connections | int | `10` | Min AMQP conns. |
| Controller.rabbit_mq.max_channels_per_connection | int | `100` | Max channels/conn. |
| Controller.redis.url | url | `redis://localhost:6379/` | Redis (optional). |
| Controller.turn.lifetime | int | `86400` | TURN cred lifetime (s). |
| Controller.turn.servers.uris | list[url] | `turn:127.0.0.1:3478?...`<br>`turns:127.0.0.1:5349?...` | TURN servers. |
| Controller.turn.servers.pre_shared_key | string | `opentalk2` | TURN PSK (if used). |
| Controller.stun.uris | list[url] | `stun:127.0.0.1:3478` | STUN servers. |
| Controller.authz.synchronize_controllers | bool | `true` | Sync across controllers. |
| Controller.call_in.tel | string | `03012345678` | PSTN dial‑in number. |
| Controller.call_in.enable_phone_mapping | bool | `false` | Map phone→user. |
| Controller.call_in.default_country_code | string | `DE` | Default country. |
| Controller.minio.uri | url | `http://minio:9000` | MinIO endpoint. |
| Controller.minio.bucket | string | `opentalk` | Bucket name. |
| Controller.minio.access_key | string | `minioadmin` | Access key. |
| Controller.minio.secret_key | secret[string] | `{{ MINIO_SECRET_KEY }}` | Secret key. |
| Controller.etcd.urls | list[url] | `etcd:2379` | etcd endpoints. |
| Controller.etherpad.url | url | `https://{{ OT_DOMAIN }}/etherpad/` | Etherpad URL (trailing `/`). |
| Controller.etherpad.api_key | secret[string] | `{{ ETHERPAD_API_KEY }}` | Etherpad API key. |
| Controller.spacedeck.url | url | `https://{{ OT_DOMAIN }}/whiteboard/` | Spacedeck URL (trailing `/`). |
| Controller.spacedeck.api_key | secret[string] | `{{ SPACEDECK_API_KEY }}` | Spacedeck API key. |
| Controller.reports.template.inline | string(multiline) | Markdown table | Inline report template. |
| Controller.reports.url | url | `http://localhost:6560` | Reports service. |
| Controller.shared_folder.provider | enum | `nextcloud` | Share provider. |
| Controller.shared_folder.url | url | `https://nextcloud.example.org/` | Provider URL. |
| Controller.shared_folder.username | string | `exampleuser` | Provider user. |
| Controller.shared_folder.password | secret[string] | `v3rys3cr3t` | Provider password. |
| Controller.shared_folder.directory | string | `opentalk/meetings` | Base directory. |
| Controller.shared_folder.expiry | int(hours) | `48` | Share expiry. |
| Controller.defaults.user_language | string | `en-US` | Default locale. |
| Controller.defaults.screen_share_requires_permission | bool | `true` | Require approval. |
| Controller.defaults.disabled_features | list[string] | `core::call_in`<br>`integration::outlook` | Disabled features. |
| Controller.endpoints.event_invite_external_email_address | bool | `true` | Allow external invites. |
| Controller.endpoints.disable_users_find | bool | `false` | Disable search. |
| Controller.endpoints.users_find_use_kc | bool | `false` | Force KC search. |
| Controller.endpoints.disallow_custom_display_name | bool | `false` | Forbid custom names. |
| Controller.endpoints.disable_openapi | bool | `false` | Disable OpenAPI. |
| Controller.metrics.allowlist | list[cidr] | `127.0.0.0/24`<br>`::ffff:0:0/96` | Metrics IP allowlist. |
| Controller.tenants.assignment | enum | `static` | Tenant assignment mode. |
| Controller.tenants.static_tenant_id | string | `OpenTalkDefaultTenant` | Static tenant id. |
| Controller.tenants.external_tenant_id_user_attribute_name | string | `tenant_id` | External attr name. |
| Controller.tariffs.assignment | enum | `static` | Tariff assignment mode. |
| Controller.tariffs.static_tariff_name | string | `OpenTalkDefaultTariff` | Static tariff. |
| Controller.tariffs.status_mapping.downgraded_tariff_name | string | `basic` | Tariff for downgraded. |
| Controller.tariffs.status_mapping.default | list[string] | `default` | Default statuses. |
| Controller.tariffs.status_mapping.paid | list[string] | `paid`<br>`all_ok` | Paid statuses. |
| Controller.tariffs.status_mapping.downgraded | list[string] | `unpaid` | Downgraded statuses. |

## Obelisk

| Parameter | Type | Default | Description |
|---|---|---|---|
| Obelisk.clock_format | string | `%d.%m.%y %X %Z` | Timestamp format. |
| Obelisk.auth.issuer | url | `{{ Keycloak.base_url }}/realms/{{ Keycloak.realm }}` | OIDC issuer. |
| Obelisk.auth.client_id | string | `Obelisk` | Client id. |
| Obelisk.auth.client_secret | secret[string] | `{{ OIDC_OBELISK_CLIENT_SECRET }}` | Client secret. |
| Obelisk.controller.domain | string | `{{ OT_DOMAIN }}/controller` | Controller base path. |
| Obelisk.controller.insecure | bool | `false` | Allow HTTP. |
| Obelisk.sip.addr | string | `{{ OT_DOMAIN }}` | SIP realm. |
| Obelisk.sip.port | int | `5060` | SIP port. |
| Obelisk.sip.id | string | `sip:alice@example.org` | SIP ID (From). |
| Obelisk.sip.contact | string | `sip:alice@127.0.0.1` | Contact URI. |
| Obelisk.sip.username | string | `0000` | SIP username. |
| Obelisk.sip.password | secret[string] | `<sipProviderPassword>` | SIP password. |
| Obelisk.sip.realm | string | `asterisk` | SIP auth realm. |
| Obelisk.sip.registrar | string | `sip:127.0.0.1` | Registrar URI. |
| Obelisk.sip.outbound_proxy | string | `sip:someotherserver.example.org` | Outbound proxy. |
| Obelisk.sip.nat_ping_delta | int | `30` | NAT keepalive (s). |
| Obelisk.sip.stun_server | string | `turn.example.com:443` | STUN/TURN server. |
| Obelisk.sip.enforce_qop | bool | `true` | Enforce qop. |
| Obelisk.sip.max_video_bitrate | int | `6000000` | Max bitrate (bps). |
| Obelisk.sip.offer_srtp | bool | `false` | Offer SRTP. |
| Obelisk.sip.rtp_port_range.start | int | `40000` | RTP start. |
| Obelisk.sip.rtp_port_range.end | int | `49999` | RTP end. |
| Obelisk.monitoring.addr | ip | `0.0.0.0` | Bind address. |
| Obelisk.monitoring.port | int | `11411` | Metrics port. |

## Mailer

| Parameter | Type | Default | Description |
|---|---|---|---|
| Mailer.rabbit_mq.url | url | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` | AMQP URL. |
| Mailer.rabbit_mq.mail_task_queue | string | `opentalk_mailer` | Queue name. |
| Mailer.smtp.smtp_server | url | `smtp://mailcrab:1025?disable_starttls=true` | SMTP URL. |
| Mailer.smtp.from_name | string | `OpenTalk - {{ OT_DOMAIN }}` | Sender name. |
| Mailer.smtp.from_email | email | `no-reply@{{ OT_DOMAIN }}` | Sender email. |
| Mailer.template_builder.join_link_builder | string | `{base_url}lobby/{room_id}` | Join link template. |
| Mailer.template_builder.guest_link_builder | string | `{base_url}/invite/{invite_id}` | Guest link template. |
| Mailer.template_builder.dashboard_event_link_builder | string | `{base_url}/dashboard/meetings/view/{event_id}` | Dashboard link template. |
| Mailer.templates.registered_invite | string | `html/txt paths` | Template mapping. |
| Mailer.templates.unregistered_invite | string | `html/txt paths` | Template mapping. |
| Mailer.templates.external_invite | string | `html/txt paths` | Template mapping. |
| Mailer.templates.registered_event_update | string | `html/txt paths` | Template mapping. |
| Mailer.templates.unregistered_event_update | string | `html/txt paths` | Template mapping. |
| Mailer.templates.external_event_update | string | `html/txt paths` | Template mapping. |
| Mailer.templates.registered_event_cancellation | string | `html/txt paths` | Template mapping. |
| Mailer.templates.unregistered_event_cancellation | string | `html/txt paths` | Template mapping. |
| Mailer.templates.external_event_cancellation | string | `html/txt paths` | Template mapping. |
| Mailer.templates.registered_uninvite | string | `html/txt paths` | Template mapping. |
| Mailer.templates.unregistered_uninvite | string | `html/txt paths` | Template mapping. |
| Mailer.templates.external_uninvite | string | `html/txt paths` | Template mapping. |
| Mailer.frontend.base_url | url | `https://livekit-test.opentalk.run` | Frontend for links. |
| Mailer.frontend.data_protection_url | url | `https://opentalk.eu/dataprotection` | Privacy link. |
| Mailer.languages.default_language | string | `de-DE` | Default locale. |
| Mailer.monitoring.addr | ip | `0.0.0.0` | Bind address. |
| Mailer.monitoring.port | int | `11411` | Metrics port. |
| Mailer.support_contact.phone | string | `+49123321123` | Support phone. |
| Mailer.support_contact.mail | email | `support@example.com` | Support email. |

## Recorder

| Parameter | Type | Default | Description |
|---|---|---|---|
| Recorder.auth.issuer | url | `{{ Keycloak.base_url }}/realms/{{ Keycloak.realm }}` | OIDC issuer. |
| Recorder.auth.client_id | string | `Recorder` | Client id. |
| Recorder.auth.client_secret | secret[string] | `{{ OIDC_RECORDER_CLIENT_SECRET }}` | Client secret. |
| Recorder.controller.domain | string | `controller:11311` | Controller URL. |
| Recorder.controller.insecure | bool | `true` | Allow HTTP. |
| Recorder.rabbitmq.uri | url | `amqp://ot:{{ RABBITMQ_PASSWORD }}@rabbitmq/%2F` | AMQP URL. |
| Recorder.rabbitmq.queue | string | `opentalk_recorder` | Work queue. |
| Recorder.recorder.display | bool | `true` | Enable display. |
| Recorder.recorder.clock_format | string | `%d.%m.%y %X %Z` | Timestamp format. |
| Recorder.recorder.hardware_acceleration.manufacturer | enum | `intel` | HW encoder vendor. |
| Recorder.recorder.hardware_acceleration.device | path | `/dev/dri/renderD129` | Device path. |
| Recorder.monitoring.port | int | `11411` | Metrics port. |

## LiveKit

| Parameter | Type | Default | Description |
|---|---|---|---|
| Livekit.port | int | `7880` | HTTP API/WS port. |
| Livekit.rtc.tcp_port | int | `7881` | TCP media port. |
| Livekit.rtc.port_range_start | int | `50000` | UDP range start. |
| Livekit.rtc.port_range_end | int | `64535` | UDP range end. |
| Livekit.rtc.use_external_ip | bool | `true` | Advertise public IP. |
| Livekit.rtc.enable_loopback_candidate | bool | `false` | Include loopback ICE. |
| Livekit.api_keys.controller | secret[string] | `{{ LIVEKIT_API_SECRET }}` | Secret for “controller”. |
| Livekit.stun_servers | list[host:port] | `turn.opentalk.run:443` | STUN servers. |
| Livekit.turn_servers[].host | string | `turn.opentalk.run` | TURN host. |
| Livekit.turn_servers[].port | int | `443` | TURN port. |
| Livekit.turn_servers[].protocol | enum | `tls` | `tls`/`tcp`/`udp`. |
| Livekit.turn_servers[].username | string | `opentalk` | TURN username. |
| Livekit.turn_servers[].credential | secret[string] | `RTJln6D...` | TURN credential. |
| Livekit.turn_embedded.enabled | bool | `false` | Embedded TURN. |
| Livekit.turn_embedded.domain | string | `meet-lk1-turn.opentalk.eu` | TURN domain. |
| Livekit.turn_embedded.tls_port | int | `5349` | TLS port. |
| Livekit.turn_embedded.udp_port | int | `3478` | UDP port. |
| Livekit.turn_embedded.external_tls | bool | `true` | TLS via proxy. |
| Livekit.prometheus_port | int | `6789` | Metrics port. |
| Livekit.logging.json | bool | `false` | JSON logs. |
| Livekit.logging.level | enum | `info` | Log level. |

## MinIO

| Parameter | Type | Default | Description |
|---|---|---|---|
| Minio.version | string | `v2` | Config version. |
| Minio.address | string | `:9000` | S3 bind. |
| Minio.rootUser | string | `minioadmin` | Root access key. |
| Minio.rootPassword | secret[string] | `{{ MINIO_ROOT_PASSWORD }}` | Root secret. |
| Minio.console_address | string | `:9001` | Console bind. |
| Minio.certs_dir | path | `.minio/certs/` | Custom certs dir. |

## RabbitMQ

| Parameter | Type | Default | Description |
|---|---|---|---|
| RabbitMQ.default_vhost | string | `/` | Default vhost. |
| RabbitMQ.default_user | string | `ot` | Default user. |
| RabbitMQ.default_pass | secret[string] | `{{ RABBITMQ_PASSWORD }}` | Default password. |
| RabbitMQ.management.path_prefix | string | `/adm/rabbitmq` | UI path prefix. |

## Traefik

| Parameter | Type | Default | Description |
|---|---|---|---|
| Traefik.basicAuth.dashboard_admin | string | `admin` | Dashboard user. |
| Traefik.basicAuth.dashboard_admin_password | secret[string] | `{{ TRAEFIK_DASHBOARD_ADMIN_PASSWORD }}` | Dashboard password. |
| Traefik.basicAuth.dashboard_admin_password_salt | secret[string] | `{{ TRAEFIK_DASHBOARD_ADMIN_PASSWORD_SALT }}` | Password salt. |
| Traefik.api.insecure | bool | `false` | No insecure API bind. |
| Traefik.log.log_level | enum | `INFO` | Log level. |

## Outlook add‑in

| Parameter | Type | Default | Description |
|---|---|---|---|
| Outlook.auth.issuer | url | `{{ Keycloak.base_url }}/realms/{{ Keycloak.realm }}` | OIDC issuer. |
| Outlook.auth.client_id | string | `Outlook` | Client id. |
| Outlook.auth.enabled | bool | `true` | Enable add‑in. |
