---
sidebar_position: 1
title: Configuration
---

# Configuring Obelisk

When Obelisk gets started, it loads the configuration from the
environment. It reads the settings in this order:

- Read environment variables which have a specific name, see section
  [Environment variables](#environment-variables).
- Load from a configuration file which defaults to `config.toml` in the current
  working directory.

## Sections in the configuration file

Functionality that can be configured through the configuration file:

- [Auth](auth.md)
- [Controller](controller.md)
- [SIP](sip.md)

## Environment variables

Settings in the configuration file can be overwritten by environment variables,
nested fields are separated by two underscores `__`. The pattern looks like
this:

```sh
OPENTALK_OBLSK_<field>__<nested-field>…
```

### Limitations

Some settings can not be overwritten by environment variables. This is for
example the case for entries in lists, because there is no environment variable
naming pattern that could identify the index of the entry inside the list.

### Examples

In order to set the `auth.client_id` field, this environment variable could be used:

```sh
OPENTALK_OBLSK_AUTH__CLIENT_ID=Obelisk
```

## Example configuration file

This file can be found in the source code distribution under `extra/example.toml`

<!-- begin:fromfile:toml:config/example.toml -->

```toml
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Obelisk OIDC authentication
#
# The Client requires a service account with the `opentalk-call-in` realm-role set.
[auth]
# OIDC issuer url
issuer = "http://localhost/auth/realms/Example"
# OIDC client ID
client_id = "Obelisk"
# OIDC client secret
client_secret = "obeliskclientsecret"

[controller]
# host-port of the controller
domain = "localhost:8000"

# Optional flag to use http/ws instead of their TLS counterparts
#insecure = false

[sip]

# Local IP address to bind to (0.0.0.0 binds to every address)
addr = "0.0.0.0"

# Port to bind to (5060 is SIP default)
port = 5060

# ID of this SIP endpoint
# When not set, it is generated as `sip:<username>@<addr>` where `addr` may be replaced by the
# public address discovered using the stun-server
#id = "sip:alice@example.org"

# Username/Password pair.
# Usually provided by the SIP provider
username = "alice"
password = "mysecurepassword"

# Realm of the given username/password pair
realm = "example.org"

# SIP URI of the registrar 
registrar = "sip:sip.example.org"

# Specify a SIP proxy to send all requests to
#outbound_proxy = "sip:someotherserver.example.org"

# Seconds between ping/pong to keep NAT binding alive
#nat_ping_delta = 30

# Host-port of the stun server used for SIP
stun_server = "stun.example.org:3478"

# Enforce quality of protection on SIP authentication
# (reuse of nonce + nonce-count instead of
#  requesting a new nonce for each request) 
# 
# Can cause compatibility issues on older registrars.
enforce_qop = true

# The port range for the SIP RTP/RTCP connections (inclusive).
#
# Defaults to 40000 - 49999
[sip.rtp_port_range]
start = 40000
end = 49999
```

<!-- end:fromfile:toml:config/example.toml -->
