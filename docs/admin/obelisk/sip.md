---
sidebar_position: 4
---

# SIP

## Supported codecs

The Obelisk supports the following Codecs:

| Codec        | Clock-Rate |
| ------------ | ---------- |
| G.711 (PCMA) | 8,000      |
| G.711 (PCMU) | 8,000      |
| G.722        | 8,000      |
| H.264        | 90,000     |

## Video Support

Obelisk includes support for SIP video calls using the **H.264** codec by default.

Most Cloud/SaaS SIP providers **do not** support video calls, so we strongly recommend verifying video capabilities with
your SIP provider.

### Cisco Call Manager

Compatibility with Cisco Unified Communications Manager (CUCM) has been tested and verified.
To enable video calls, **Early Offer** must be enabled in the SIP Profile on CUCM.

### Cisco RoomKit

Cisco RoomKit devices have been tested and verified to work with Obelisk, both via Asterisk and CUCM.

## Configuration

The section in the [configuration file](configuration.md) is called `sip`.

| Field                          | Type                              | Required | Default value | Description                                                                                                                                                                                                                                          |
| ------------------------------ | --------------------------------- | -------- | ------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `addr`                         | `string`                          | yes      | -             | The local IP address to bind to (`0.0.0.0` binds to every address)                                                                                                                                                                                   |
| `port`                         | `int`                             | yes      | -             | The port to bind to (usually `5060`)                                                                                                                                                                                                                 |
| `inbound_addr`                 | `string`                          | no       | none          | Set the inbound address to use in SIP and SDP messages in the format `<ip>:<port>`, e.g. `192.0.2.1:5060`. Should only be used in NAT scenarios where using a STUN server is not possible. This value is ignored when a `stun_server` is configured. |
| `inbound_media_ip`             | `string`                          | no       | none          | Similar to `inbound_addr` except constrained to just SDP. Overrides the value of `inbound_addr` for SDP. Should be used to set the inbound IP for media (RTP/RTCP).                                                                                  |
| `transport`                    | `string`                          | no       | `udp_tcp`     | Defines which transports to bind on the local `addr` and `port`, possible options are `udp`, `tcp` and `udp_tcp`.                                                                                                                                    |
| `id`                           | `string`                          | no       | See below     | The ID of this SIP endpoint in the format `sip:<username>@<host>`, used for creating the From/To header in SIP REGISTER messages                                                                                                                     |
| `contact`                      | `string`                          | no       | See below     | The Contact of this SIP endpoint in the format `sip:<username>@<host>`, used for creating the SIP Contact header                                                                                                                                     |
| `username`                     | `string`                          | no       | none          | The username to register with the SIP provider                                                                                                                                                                                                       |
| `password`                     | `string`                          | no       | none          | The password to register with the SIP provider                                                                                                                                                                                                       |
| `realm`                        | `string`                          | no       | none          | The realm of the given username/password pair                                                                                                                                                                                                        |
| `registrar`                    | `string`                          | no       | none          | The SIP URI of the registrar in the format `sip:<domain>`                                                                                                                                                                                            |
| `outbound_proxy`               | `string`                          | no       | none          | The SIP proxy to send all requests to in the format `sip:<domain>`                                                                                                                                                                                   |
| `nat_ping_delta`               | `string`                          | no       | 30 seconds    | Seconds between ping and pong to keep the NAT binding alive                                                                                                                                                                                          |
| `stun_server`                  | `string`                          | no       | none          | The host and optional port of the STUN server in the format `<host>[:<port>]`                                                                                                                                                                        |
| `enforce_qop`                  | `bool`                            | no       | `false`       | `true` to enforce quality of protection on SIP authentication                                                                                                                                                                                        |
| `offer_srtp`                   | `bool`                            | no       | `false`       | `true` to offer SRTP as media transport when obelisk is creating the initial SDP offer                                                                                                                                                               |
| `rtp_port_range`               | [RTP port range](#rtp-port-range) | no       | 40000-49999   | The port range for the SIP RTP/RTCP connections                                                                                                                                                                                                      |
| `max_video_bitrate`            | `int`                             | no       | 6000000       | Maximum allowed video bitrate to the caller in bits/s                                                                                                                                                                                                |
| `encode_video_at_half_bitrate` | `bool`                            | no       | `false`       | Encode video at half the bitrate advertised by the caller. This ensures compatibility with devices that incorrectly report their supported H.264 maximum bitrate for both sending and receiving, instead of just receiving.                          |
| `display_name_source`          | `string`                          | no       | `Contact`     | Which header to infer the caller's display name from. Supported values are `Contact` and `From`.                                                                                                                                                     |

If `ìd` is not set, it is generated as `sip:<username>@<registrar-host>`. If no
registrar is configured it is generated just like the `contact` field.

If `contact` is not set, it is generated as `sip:<username>@<addr>` where
`<addr>` may be replaced by the public address discovered using the STUN server.

### RTP port range

| Field   | Type     | Required | Default value | Description                                                        |
| ------- | -------- | -------- | ------------- | ------------------------------------------------------------------ |
| `start` | `string` | yes      | -             | The lower bound of the port range for the SIP RTP/RTCP connections |
| `end`   | `string` | yes      | -             | The upper bound of the port range for the SIP RTP/RTCP connections |

### Example

```toml
[sip]
addr = "0.0.0.0"
port = 5060
id = "sip:alice@example.org"
contact = "sip:alice@192.168.1.100"
username = "user"
password = "pass"
realm = "asterisk"
registrar = "sip:sip.example.org"
stun_server = "stun.example.org:3478"

[sip.rtp_port_range]
start = 40000
end = 49999
```
