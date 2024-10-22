{{- with .Values }}
{{- $defaultProtocol := .global.defaultWebServiceProtocol }}
{{- $dbPassword := .global.controllerDB.password }}
{{- with .controller -}}
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

[logging]
{{- with .logging }}
{{- with .defaultDirectives }}
# Default tracing directives that will always be applied after RUST_LOG's directives.
# Each array entry may contain a single directive.
# Below are some example directives which are used by default to reduce the amount of spamming some crates do by default.
# The defaults will always apply, but they can be explicitly overwritten with this config or the RUST_LOG environment
# variable. The priority of the different configuration options is: RUST_LOG > config file > defaults

default_directives = [
{{- $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]
{{- end }}
{{- if .otlpTracingEndpoint }}
# Specify an optional OTLP tracing endpoint to export traces to
otlp_tracing_endpoint = "{{ .otlpTracingEndpoint }}"{{ end }}
{{- with .serviceName }}
# Service name when using opentelemetry
serviceName = "{{ . }}"{{ end }}
{{- with .serviceNameSpace }}
# Service namespace when using opentelemetry
service_namespace = "{{ . }}"{{ end }}
{{- with .serviceInstanceId }}
# Service instance id when using opentelemetry. A random UUID will be generated at runtime if not set here.
service_instance_id = "{{ . }}"{{ end }}
{{- end }}

[database]
# URL used to connect to a postgres.
{{- with .database }}
{{- if .url }}
url = "{{ .url }}"
{{- else if and .user $dbPassword .host .port .name }}
url = "postgres://{{ .user }}:{{ $dbPassword }}@{{ .host }}:{{ .port }}/{{ .name }}"
{{- else }}{{ fail ".controller.database.url or url components not set!" }}{{ end }}
{{- with .maxConnections }}
# Maximum number of connections allowed to the server.
# Defaults to 100 which is the default of postgres.
max_connections = {{ . }}{{ end }}
{{ end }}

{{- with .http }}
[http]
{{- with .port }}
# The port to bind the HTTP Server to (defaults to 11311).
port = {{ . }}{{ end }}

{{- with .tls }}
{{- if and .certificatePath .privateKeyPath }}
[http.tls]
certificate = "{{ .certificatePath }}"
private_key = "{{ .privateKeyPath }}"
{{- end }}
{{- end }}
{{- end }}

[keycloak]
# Settings for the keycloak which is the user provider
# and allows authentication via OIDC
{{- with $.Values.global.keycloak }}

{{- if .baseUrl }}
base = "{{ .baseUrl }}"
{{- else }}
base_url = "{{ .protocol | default $defaultProtocol }}://{{ .domain }}{{ .path }}{{ .basePath }}"
{{- end }}
realm = "{{ .realm }}"
# OIDC Client ID.
client_id = "{{ .oidcClientId }}"
# OIDC Client secret (application requires confidential client).
{{- if .oidcClientSecret }}
client_secret = "{{ .oidcClientSecret }}"
{{- else }}{{ fail ".global.keycloak.oidcClientSecret is not set!" }}{{ end }}
{{- end }}
{{- with .keycloak.externalIdUserAttributeName }}
external_id_user_attribute_name = "{{ . }}"{{ end }}

{{- with $.Values.global.livekit }}

[livekit]
{{- if .url }}
url = "{{ .url }}"
{{- else }}
url = "{{ .protocol | default $defaultProtocol }}://{{ .domain }}{{ .path }}"
{{- end }}
{{- if and .apiKey .apiSecret }}
api_key = "{{ .apiKey }}"
api_secret = "{{ .apiSecret }}"
{{- else }}{{ fail ".global.livekit.apiSecret or .global.livekit.apiSecret is not set!" }}{{ end }}
{{- end }}

{{- with .roomServer }}

{{ if not $.Values.global.livekit }}
[room_server]
{{- with .maxVideoBitrate }}
# Maximum bitrate allowed for media sessions that will be used to transmit webcam video/audio
# Example: 1.5 Mbit/s
# max_video_bitrate = "1500000"
max_video_bitrate = "{{ . | int }}"{{ end }}
{{- with .maxScreenBitrate }}
# Maximum bitrate allowed for media sessions used for screen share
# Example: 1MB/s
# max_screen_bitrate = "8000000"
max_screen_bitrate = "{{ . | int }}"{{ end }}
{{- with .speakerFocusPackets}}
# Number of packets with with given `speaker_focus_level`
# needed to detect a speaking participant.
#
# Default: 50 packets (1 second of audio)
speaker_focus_packets = "{{ . }}"{{ end }}
{{- with .speakerFocusLevel }}
# Average value of audio level needed per packet.
#
# min: 127 (muted)
# max: 0   (loud)
# default: 50
# speaker_focus_level = "50"
speaker_focus_level = "{{ . }}"{{ end }}

# Connection settings for the channel used to talk to the room server.
# Currently these should be equal to the settings in janus.transport.rabbitmq.jcfg
# of the respective janus instance.

{{- range .connections }}
[[room_server.connections]]
to_routing_key = "{{ .toRoutingKey }}"
exchange = "{{ .exchange }}"
from_routing_key = "{{ .fromRoutingKey }}"
{{ end }}
{{- end }}

{{- with $.Values.global.rabbitmq }}
[rabbit_mq]
# The URL to use to connect to the rabbit mq broker
{{- if .url }}
url = "{{ .url }}"
{{- else if and .user .password .host .port }}
url = "amqp://{{ .user }}:{{ .password }}@{{ .host }}:{{ .port }}{{ .path }}"
{{- else }}{{ fail ".global.rabbitmq.url or url components not set!" }}{{ end }}
{{- with .mailTaskQueue }}
# The rabbitmq queue name for the mail worker,
# mailing is disabled when this is not set.
mail_task_queue = "{{ . }}"{{ end }}
{{- with .recordingTaskQueue }}
# The rabbitmq queue name for the recording worker,
# recording is disabled when this is not set.
recording_task_queue = "{{ . }}"{{ end }}
{{- end }}
{{- end }}

{{- with .rabbitmq }}
{{- with .minConnections }}
# Minimum amount of connections to retain when removing stale connections
min_connections = {{ . }}{{ end }}
{{- with .maxChannelsPerConnection }}
# Maximum number of amqp channels per connection
max_channels_per_connection = {{ . }}{{ end }}
{{ end }}

{{- with .redis }}
[redis]
# Redis URL used to connect the redis server
{{- if .url }}
url = "{{ .url }}"
{{- else }}
url = "redis://{{ .host }}:{{ .port }}{{ .path }}"{{ end }}
{{- end }}

{{- with .turn }}
[turn]
{{- with .lifetime }}
# Lifetime of the generated credentials (in seconds)
lifetime = {{ . | int }}{{ end }}
{{ range .servers }}
[[turn.servers]]
{{- with .uris }}
# URIS of this Turn Server following rfc7065
uris = [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]{{ end }}
{{- if .preSharedKey }}
# The Pre Shared Key set with --static-auth-secret=...
pre_shared_key = "{{ .preSharedKey }}"
{{- else }}{{ fail ".controller.turn.servers.preSharedKey not set!" }}{{ end }}
{{- end }}

{{- end }}
{{- with .stun }}
[stun]
{{- with .uris }}
uris = [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]{{ end }}

{{- end }}
{{- with .authz }}
[authz]
{{- with .synchronizeControllers }}
synchronize_controllers = "{{ . }}"{{ end }}
{{- end }}

{{- with .callIn }}
{{- if and .tel .enablePhoneMapping .defaultCountryCode }}
[call_in]
# Set a phone number which will be displayed to the user
# for the call-in service
tel="{{ .tel }}"
# Enable the mapping of user names to their phone number. This requires
# the OIDC provider to have a phone number field configured for their users.
enable_phone_mapping = "{{ .enablePhoneMapping }}"

# The default country code for call in numbers. Notated in Alpha-2 code (ISO 3166)
# Phone numbers that do not fall in the category of the default country must be notated
# in the international format.
default_country_code = "{{ .defaultCountryCode }}"

{{- else }}{{ fail ".controller.callIn variables not set!" }}{{ end }}
{{- end }}

{{- with $.Values.global.minio }}
[minio]
{{- if .uri }}
uri = "{{ .uri }}"
{{- else if and .domain .port }}
uri = "{{ .protocol | default $defaultProtocol }}://{{ .domain }}:{{ .port }}{{ .path }}"
{{- else }}{{ fail ".global.minio.url or url components not set!" }}{{ end }}
{{ if .bucket }}
bucket = "{{ .bucket }}"
{{- else }}{{ fail ".global.minio.bucket not set!" }}{{ end }}
{{ if and .accessKey .secretKey }}
access_key = "{{ .accessKey }}"
secret_key = "{{ .secretKey }}"
{{ else }}{{ fail ".global.minio.accessKey or .secretKey not set!" }}{{ end }}

{{- end }}
{{- with .etcd }}
# Etcd configuration
[etcd]
{{- with .urls }}
# A list urls of a etcd cluster
urls = [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]
{{- end }}

{{- end }}
{{- with $.Values.global.etherpad }}
# The etherpad configuration for the protocol module
[etherpad]
{{- if .url }}
url = "{{ .url }}"
{{- else if .domain}}
url = "{{ .protocol | default $defaultProtocol }}://{{ .domain }}{{ .path }}"
{{- else }}{{ fail ".global.etherpad.domain not set!" }}{{ end }}
{{- if .apiKey }}
api_key = "{{ .apiKey }}"
{{- else }}{{ fail ".global.etherpad.apiKey not set!" }}{{ end }}
{{ end }}

{{- with $.Values.global.spacedeck }}
[spacedeck]
{{- if .url }}
url = "{{ .url }}"
{{- else if .domain}}
url = "{{ .protocol | default $defaultProtocol }}://{{ .domain }}{{ .path }}"
{{- else }}{{ fail ".global.spacedeck.domain not set!" }}{{ end }}
{{- if .apiKey }}
api_key = "{{ .apiKey }}"

{{- else }}{{ fail ".global.spacedeck.apiKey not set!" }}{{ end }}
{{ end }}

{{- with .reports }}
# Reports configuration
[reports]
{{- if .url }}
url = "{{ .url }}"
{{- else }}
url = "{{ .protocol | default $defaultProtocol }}://{{ .domain }}{{- if .port }}:{{ .port }}{{- end }}{{ .path }}"
{{- end }}
# A template can be provided. If no template is provided the built-in template will be used.
 {{- with .template.inline }}
template.inline = """{{ . }}
"""
{{- end }}
{{ end }}

{{- with .sharedFolder }}
{{- if .provider }}
# Shared folder configuration
[shared_folder]
provider = "{{ .provider }}"
{{- if .url }}
url = "{{ .url }}"
{{- else if .domain }}
url = "{{ .protocol | default $defaultProtocol }}://{{ .domain }}{{ .path }}"
{{- else }}{{ fail ".controller.sharedFolder.domain or url not set!" }}{{ end }}
{{- if and .username .password }}
username = "{{ .username }}"
password = "{{ .password }}"
{{- else }}{{ fail ".controller.sharedFolder.username or .password not set!" }}{{ end }}
{{- with .directory }}
# Optional subdirectory under the user's folder
directory = "{{ . }}"{{ end }}
{{- with .expiryDays }}
# Optional expiry of folder shares in days
expiry = {{ . }}{{ end }}
{{- end }}
{{ end }}

{{- with .defaults }}
# Default/fallback values
[defaults]
{{- with .languageTag }}
# Default language of a new user
user_language = "{{ . }}"{{ end }}
{{- with .screenShareRequiresPermission }}
# Default presenter role for all users (defaults to false if not set)
screen_share_requires_permission = {{ . }}{{ end }}
{{- with .disabledFeatures }}
# A list of disabled features in the controller. By default all features are enabled.
# Format: <module>::<feature>. A missing module defaults to "core".
# Currently supported features:
# - core::call_in
# - integration::outlook
disabled_features = [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]{{ end }}
{{ end }}

{{- with .endpoints }}
# Settings for endpoints
[endpoints]
{{- with .disableUsersFind }}
# Disable the /users/find endpoint for performance or privacy reasons
disable_users_find = {{ . }}{{ end }}
{{- with .usersFindUseKc }}
# Enable user-searching using keycloak's admin API
# This allows for finding users which have not yet
# logged into the controller
users_find_use_kc = {{ . }}{{ end }}
{{- with .eventInviteExternalEmailAddress }}
# Allow inviting any unchecked email address.
# Not recommended without proper outgoing anti-spam protection
event_invite_external_email_address = {{ . }}{{ end }}
{{- with .disallowCustomDisplayName }}
# Prohibit users from changing the display name (guests are always allowed to change it)
disallow_custom_display_name = {{ . }}{{ end }}
{{- with .disableOpenapi }}
# Disable the OpenAPI endpoint under `/v1/openapi.json` and the corresponding
# swagger endpoint under `/swagger`.
disable_openapi = {{ . }}{{ end }}
{{ end }}

{{- with .metrics }}
# Configuration for the /metric HTTP endpoint
[metrics]
{{- with .allowList }}
# Allowlist for the /metrics endpoint
#
# Example: Allow all traffic from localhost
#allowlist = ["192.168.0.0/16", "172.16.0.0/12", "::/1"]
allowlist = [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]{{ end }}
{{ end }}

{{- with .tenants }}
[tenants]
# Configure how users are assigned to tenants
# The following assignment strategies are available:
#   - "static" (default):
#     Every user is assigned to a single tenant with a tenant_id specified in the static_tenant_id field.
#     static_tenant_id's default value is "OpenTalkDefaultTenant".
#   - "by_external_tenant_id":
#     The OIDC provider (Keycloak) must be configured to include a "tenant_id" field in its
#     id_token's JWT claims. It is used to assign users to the correct tenant.
#     The OIDC provider must also provide a user attribute holding the tenant_id. The name of this
#     field is set here in external_tenant_id_user_attribute_name, the default field name is "tenant_id".
#
{{- if eq .assignment "static" }}
# Static assignment (Default configuration if nothing is specified):
assignment = "{{ .assignment }}"
{{- with .staticTenantId }}
static_tenant_id = "{{ . }}"{{ end }}
{{- else if eq .assignment "by_external_tenant_id" }}
# Assignment by JWT tenant_id:
assignment = "{{ .assignment }}"
{{- with .externalTenantIdUserAttributeName }}
external_tenant_id_user_attribute_name = "{{ . }}"{{ end }}
{{- end }}
{{ end }}

{{- with .tariffs }}
[tariffs]
# Configure how tariffs are assigned to users
# The following assignment strategies are available:
#   - "static" (default):
#      Every user is assigned the same tariff with the tariff's name specified in a separate field
#     called static_tariff_name. The default value is "OpenTalkDefaultTariff".
#   - "by_external_tariff_id":
#      The OIDC provider (Keycloak) must be configured to include a "tariff_id" field it's
#     id_token's JWT claims. It is used to assign users the correct tariff.
#
{{ if eq .assignment "static" }}
# Static Example (Default configuration if nothing is specified)
assignment = "{{ .assignment }}"
{{- with .staticTariffName }}
static_tariff_name = "{{ . }}"{{ end }}
{{- else if eq .assignment "by_external_tariff_id" }}
# Assignemnt by JWT tariff_id example:
"assignment = "{{ .assignment }}"
#
# Status mapping for tariff status. Can only be used if the tariff assignment
# is configured as "by_external_tariff_id". If present, the controller will look
# at the JWT attribute named "tariff_status" and transfer its value to its
# internal tariff status based on the values of the "default", "paid" and
# "downgraded" field values. An entry in any of the lists below must be unique
# across all lists. For example, if the paid list contains "all_ok", then
# "all_ok" must not appear in any other list.
{{- with .statusMapping }}
[tariffs.status_mapping]
{{- with .downgradedTariffName }}
# The name of the tariff that gets applied when the user's tariff state is "downgraded".
downgraded_tariff_name = "{{ . }}"{{ end }}
{{- with .default }}
# The default tariff status, usually a tariff that any user can get without paying
# Any user with an invalid value in the "tariff_status" attribute will be set to
# the default status, but a warning will be issued if the mapping does not contain
# that attribute value.
default = [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]{{ end }}
{{- with .paid }}
# The user's tariff has been paid and is valid.
paid = [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]{{ end }}
{{- with .downgraded }}
# The user has booked a specific tariff, but is not allowed to use it, e.g. because
# it is unpaid. Therefore the user's tariff is downgraded to the fallback tariff.
downgraded = [
{{ $allEntries := . }}{{ range $index, $entry := . }}
  "{{ $entry }}"{{ if lt $index (add (len $allEntries) -1) }},{{ end }}{{ end }}
]{{ end }}
{{- end }}
{{- end }}
{{ end }}

{{- end }}{{/* close 'with .controller' */}}
{{- end }}{{/* close 'with .Values' */}}