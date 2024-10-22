{{- with .Values }}
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Obelisk OIDC authentication
#
# The Client requires a service account with the `opentalk-call-in` realm-role set.
[auth]
# OIDC issuer url
issuer = "{{ .obelisk.auth.issuer}}"
# OIDC client ID
client_id = "{{ .obelisk.auth.clientId}}"
# OIDC client secret
client_secret = "{{ .obelisk.auth.clientSecret}}"

[controller]
# host-port of the controller
domain = "{{ .obelisk.controller.domain}}"

# Optional flag to use http/ws instead of their TLS counterparts
#insecure = false

[sip]

# Local IP address to bind to (0.0.0.0 binds to every address)
addr = "{{ .obelisk.sip.addr}}"

# Port to bind to (5060 is SIP default)
port = {{ .obelisk.sip.port}}

# ID of this SIP endpoint
# When not set, it is generated as sip:<username>@<addr> where `addr` may be replaced by the
# public address discovered using the stun-server
#id = "{{ .obelisk.sip.id}}"

# Username/Password pair.
# Usually provided by the SIP provider
username = "{{ .obelisk.sip.username}}"
password = "{{ .obelisk.sip.password}}"

# Realm of the given username/password pair
realm = "{{ .obelisk.sip.realm}}"

# SIP URI of the registrar 
registrar = "{{ .obelisk.sip.registrar}}"

# Seconds between ping/pong to keep NAT binding alive
#nat_ping_delta = {{ .obelisk.sip.username}}

# Host-port of the stun server used for SIP
#stun_server = "{{ .obelisk.sip.natPingDelta}}"

# Enforce quality of protection on SIP authentication
# (reuse of nonce + nonce-count instead of
#  requesting a new nonce for each request) 
# 
# Can cause compatibility issues on older registrars.
enforce_qop = {{ .obelisk.sip.enforceQop}}

# The port range for the SIP RTP/RTCP connections (inclusive).
#
# Defaults to 40000 - 49999
[sip.rtp_port_range]
start = {{ .obelisk.sipRtpPortRange.start}}
end = {{ .obelisk.sipRtpPortRange.end}}
{{- end }}