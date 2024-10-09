# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

[logging]
# Default tracing directives that will always be applied after RUST_LOG's directives.
# Each array entry may contain a single directive.
# Below are some example directives which are used by default to reduce the amount of spamming some crates do by default.
# The defaults will always apply, but they can be explicitly overwritten with this config or the RUST_LOG environment
# variable. The priority of the different configuration options is: RUST_LOG > config file > defaults

default_directives = [
{{- range $index, $directive := .Values.controller.logging.defaultDirectives }}
    "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.logging.defaultDirectives) -1) }},{{ end }}
{{- end }}
]

{{ if .Values.controller.logging.otlpTracingEndpoint }}
# Specify an optional OTLP tracing endpoint to export traces to
otlp_tracing_endpoint = "{{ .Values.controller.logging.otlpTracingEndpoint }}"
{{ end }}

{{- if .Values.controller.logging.serviceName -}}
# Service name when using opentelemetry
serviceName = "{{ .Values.controller.logging.serviceName }}"
{{ end }}
{{- if .Values.controller.logging.serviceNameSpace -}}
# Service namespace when using opentelemetry
service_namespace = "{{ .Values.controller.logging.serviceNameSpace }}"
{{ end }}
{{- if .Values.controller.logging.serviceInstanceId -}}
# Service instance id when using opentelemetry. A random UUID will be generated at runtime if not set here.
service_instance_id = "{{ .Values.controller.logging.serviceInstanceId }}"
{{ end }}

[database]
{{- if .Values.controller.database.url }}
# URL used to connect to a postgres.
url = "{{ .Values.controller.database.url }}"
{{ end }}
{{- if .Values.controller.database.maxConnections -}}
# Maximum number of connections allowed to the server.
# Defaults to 100 which is the default of postgres.
max_connections = "{{ .Values.controller.database.maxConnections }}"
{{ end }}

[http]
# The port to bind the HTTP Server to (defaults to 11311).
port = {{ .Values.controller.http.port }}

# Settings for the keycloak which is the user provider
# and allows authentication via OIDC

cors.allowed_origin = [
{{- range $index, $directive := .Values.controller.http.corsAllowedOrigin }}
    "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.http.corsAllowedOrigin) -1) }},{{ end }}
{{- end }}
]

[keycloak]
# URL to the keycloak
base_url = "{{ .Values.controller.keycloak.baseUrl }}"
# Name of the keycloak realm
realm = "{{ .Values.controller.keycloak.realm }}"
# Client ID
client_id = "{{ .Values.controller.keycloak.clientId }}"
# Client secret (application requires confidential client).
client_secret = "{{ .Values.global.keycloak.clientSecret }}"

# Configure how Keycloak users resulting from a search and registered Opentalk users are assigned to each other
# The following assignment strategies are available:
#   - by Keycloak id (default): Keycloak users are assigned to Opentalk users using Keycloak's id field.
#   - by user attribute: Keycloak must provide a user attribute holding the user IDs. The name of this user
#                        attribute must be set here in external_id_user_attribute_name.
#external_id_user_attribute_name = "{{ .Values.controller.keycloak.externalIdUserAttributeName }}"

[livekit]
url = "{{ .Values.controller.livekit.url }}"

api_key = "{{ .Values.controller.livekit.apiKey }}"
api_secret = "{{ .Values.controller.livekit.apiSecret }}"

[rabbit_mq]
# The URL to use to connect to the rabbit mq broker
url = "{{ .Values.controller.rabbitmq.url }}"

# The rabbitmq queue name for the mail worker,
# mailing is disabled when this is not set.
mail_task_queue = "{{ .Values.controller.rabbitmq.mailTaskQueue }}"

# The rabbitmq queue name for the recorder,
# recording is disabled when this is not set.
recording_task_queue = "{{ .Values.controller.rabbitmq.recordingTaskQueue }}"

# Minimum amount of connections to retain when removing stale connections
min_connections = {{ .Values.controller.rabbitmq.minConnections }}

# Maximum number of amqp channels per connection
max_channels_per_connection = {{ .Values.controller.rabbitmq.maxChannelsPerConnection }}

[redis]
# Configuration of a redis server which can be used for synchronizing multiple
# controllers running in a cluster to provide an OpenTalk web api and meeting
# signaling service.
#
# If this section is present, then the redis service will be used for
# synchronizing meeting state throughout all controllers in the cluster. If it
# is left out entirely, then the service will run in "standalone" mode.
#
# Redis URL used to connect the redis server
url = "{{ .Values.controller.redis.url }}"

[turn]
# Lifetime of the generated credentials (in seconds)
lifetime = {{ .Values.controller.turn.lifetime }}

[[turn.servers]]
# URIS of this Turn Server following rfc7065
uris = [
{{- range $index, $directive := .Values.controller.turn.servers.uris }}
    "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.turn.servers.uris) -1) }},{{ end }}
{{- end }}
]
# The Pre Shared Key set with --static-auth-secret=...
pre_shared_key = "{{ $.Values.controller.turn.preSharedKey }}"

[stun]
uris = [
{{- range $index, $directive := .Values.controller.stun.uris }}
    "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.stun.uris) -1) }},{{ end }}
{{- end }}
]

[authz]
# Should the controller publish/receive ACL changes via RabbitMQ to/from other controllers
synchronize_controllers = {{ .Values.controller.authz.synchronizeControllers }}

[call_in]
# Set a phone number which will be displayed to the user
# for the call-in service
tel="{{ .Values.controller.callIn.tel }}"
# Enable the mapping of user names to their phone number. This requires
# the OIDC provider to have a phone number field configured for their users.
enable_phone_mapping={{ .Values.controller.callIn.enablePhoneMapping }}
# The default country code for call in numbers. Notated in Alpha-2 code (ISO 3166)
# Phone numbers that do not fall in the category of the default country must be notated
# in the international format.
default_country_code="{{ .Values.controller.callIn.defaultCountryCode }}"

# MinIO configuration
[minio]
# The URI to the MinIO instance
uri = "{{ .Values.controller.minio.uri }}"
# Name of the bucket
bucket = "{{ .Values.controller.minio.bucket }}"
# Access key for the MinIO bucket
access_key = "{{ .Values.controller.minio.accessKey }}"
# Secret key for the MinIO bucket
secret_key = "{{ .Values.controller.minio.secretKey }}"


{{ if .Values.controller.etcd.urls }}
# Etcd configuration
[etcd]
# A list urls of a etcd cluster
urls = [
{{- range $index, $directive := .Values.controller.etcd.urls }}
    "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.etcd.urls) -1) }},{{ end }}
{{- end }}
]
{{ end }}

{{ if .Values.controller.etherpad.url }}
# The etherpad configuration for the meeting-notes module
[etherpad]
url = "{{ .Values.controller.etherpad.url }}"
# Etherpads api key
api_key = "{{ .Values.controller.etherpad.apiKey }}"
{{ end }}
 
{{ if .Values.controller.spacedeck.url }}
# Spacedeck configuration
[spacedeck]
url = "{{ .Values.controller.spacedeck.url }}"
api_key = "{{ .Values.controller.spacedeck.apiKey }}"
{{ end }}

### # Shared folder configuration
### [shared_folder]
### provider = "{{ .Values.controller.sharedFolder.provider }}"
### url = "{{ .Values.controller.sharedFolder.url }}"
### username = "{{ .Values.controller.sharedFolder.username }}"
### password = "{{ .Values.controller.sharedFolder.password }}"
### # Optional subdirectory under the user's folder
### #directory = "{{ .Values.controller.sharedFolder.directory }}"
### # Optional expiry of folder shares in days
### #expiry = {{ .Values.controller.sharedFolder.expiry }}
### 
### # Default/fallback values
### [defaults]
### # Default language of a new user
### user_language = "{{ .Values.controller.defaults.userLanguage }}"
### # Default presenter role for all users (defaults to false if not set)
### screen_share_requires_permission = {{ .Values.controller.defaults.screenShareRequiresPermission }}
### # A list of disabled features in the controller. By default all features are enabled.
### # Format: <module>::<feature>. A missing module defaults to "core".
### # Currently supported features:
### # - core::call_in
### # - integration::outlook
### disabled_features = [
### {{- range $index, $directive := .Values.controller.defaults.disabledFeatures }}
###     "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.defaults.disabledFeatures) -1) }},{{ end }}
### {{- end }}
### ]
### 
### # Settings for endpoints
### [endpoints]
### # Disable the /users/find endpoint for performance or privacy reasons
### disable_users_find = {{ .Values.controller.endpoints.disableUsersFind }}
### 
### # Enable user-searching using keycloak's admin API
### # This allows for finding users which have not yet
### # logged into the controller
### users_find_use_kc = {{ .Values.controller.endpoints.usersFindUseKc }}
### 
### # Allow inviting any unchecked email address.
### # Not recommended without proper outgoing anti-spam protection
### event_invite_external_email_address = {{ .Values.controller.endpoints.eventInviteExternalEmailAddress }}
### 
### # Prohibit users from changing the display name (guests are always allowed to change it)
### disallow_custom_display_name = {{ .Values.controller.endpoints.disallowCustomDisplayName }}
### 
### # Disable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
### # swagger endpoint under `/swagger`.
### disable_openapi = {{ .Values.controller.endpoints.disableOpenapi }}
### 
### # Configuration for the /metrics HTTP endpoint
### [metrics]
### # Allowlist for the /metrics endpoint
### #
### # Example: Allow all traffic from localhost
### allowlist = [
### {{- range $index, $directive := .Values.controller.metrics.allowlist }}
###     "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.metrics.allowlist) -1) }},{{ end }}
### {{- end }}
### ]
### 
### [tenants]
### # Configure how users are assigned to tenants
### # The following assignment strategies are available:
### #   - "static" (default): Every user is assigned to a single tenant with a tenant_id specified in the static_tenant_id field.
### #                         static_tenant_id's default value is "OpenTalkDefaultTenant".
### #   - "by_external_tenant_id": The OIDC provider (Keycloak) must be configured to include a "tenant_id" field in its
### #                              id_token's JWT claims. It is used to assign users to the correct tenant.
### #                              The OIDC provider must also provide a user attribute holding the tenant_id. The name of this
### #                              field ist set here in external_tenant_id_user_attribute_name, the default field name is "tenant_id".
### #
### # Static assignment (Default configuration if nothing is specified):
### {{ if eq .Values.controller.tenants.assignment "static" }}
### assignment = "static"
### static_tenant_id = "{{ .Values.controller.tenants.staticTenantId }}"
### {{ end }}
### #
### # Assignment by JWT tenant_id:
### # assignment = "by_external_tenant_id"
### {{ if eq .Values.controller.tenants.assignment "by_external_tariff_id" }}
### assignment = "by_external_tariff_id"
### external_tenant_id_user_attribute_name = "{{ .Values.controller.tenants.externalTenantIdUserAttributeName }}"
### {{ end }}
### 
### [tariffs]
### # Configure how tariffs are assigned to users
### # The following assignment strategies are available:
### #   - "static" (default): Every user is assigned the same tariff with the tariff's name specified in a separate field
### #               called static_tariff_name. The default value is "OpenTalkDefaultTariff".
### #   - "by_external_tariff_id": The OIDC provider (Keycloak) must be configured to include a "tariff_id" field it's
### #                              id_token's JWT claims. It is used to assign users the correct tariff.
### #
### # Static Example (Default configuration if nothing is specified)
### {{ if eq .Values.controller.tariffs.assignment "static" }}
### assignment = "static"
### static_tariff_name = "{{ .Values.controller.tariffs.staticTariffName }}"
### {{ end }}
### #
### # Assignemnt by JWT tariff_id example:
### {{ if eq .Values.controller.tariffs.assignment "by_external_tariff_id" }}
### assignment = "by_external_tariff_id"
### #
### # Status mapping for tariff status. Can only be used if the tariff assignment
### # is configured as "by_external_tariff_id". If present, the controller will look
### # at the JWT attribute named "tariff_status" and transfer its value to its
### # internal tariff status based on the values of the "default", "paid" and
### # "downgraded" field values. An entry in any of the lists below must be unique
### # across all lists. For example, if the paid list contains "all_ok", then
### # "all_ok" must not appear in any other list.
### [tariffs.status_mapping]
### #
### # The name of the tariff that gets applied when the user's tariff state is "downgraded".
### downgraded_tariff_name = "{{ .Values.controller.tariffs.downgradedTariffName }}"
### #
### # The default tariff status, usually a tariff that any user can get without paying
### # Any user with an invalid value in the "tariff_status" attribute will be set to
### # the default status, but a warning will be issued if the mapping does not contain
### # that attribute value.
### default = [
### {{- range $index, $directive := .Values.controller.tariffs.tariffsStatusMapping.default }}
###     "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.tariffs.tariffsStatusMapping.default) -1) }},{{ end }}
### {{- end }}
### ]
### #
### # The user's tariff has been paid and is valid.
### paid = [
### {{- range $index, $directive := .Values.controller.tariffs.tariffsStatusMapping.paid }}
###     "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.tariffs.tariffsStatusMapping.paid) -1) }},{{ end }}
### {{- end }}
### ]
### #
### # The user has booked a specific tariff, but is not allowed to use it, e.g. because
### # it is unpaid. Therefore the user's tariff is downgraded to the fallback tariff.
### downgraded = [
### {{- range $index, $directive := .Values.controller.tariffs.tariffsStatusMapping.downgraded }}
###     "{{ $directive }}"{{ if lt $index (add (len $.Values.controller.tariffs.tariffsStatusMapping.downgraded) -1) }},{{ end }}
### {{- end }}
### ]
### {{ end }}