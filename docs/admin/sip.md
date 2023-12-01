---
sidebar_position: 4
---

# SIP

## Configuration

The section in the [configuration file](configuration.md) is called `sip`.

| Field            | Type                              | Required | Default value | Description                                                                   |
| ---------------- | --------------------------------- | -------- | ------------- | ----------------------------------------------------------------------------- |
| `addr`           | `string`                          | yes      | -             | The local IP address to bind to (`0.0.0.0` binds to every address)            |
| `port`           | `int`                             | yes      | -             | The port to bind to (usually `5060`)                                          |
| `id`             | `string`                          | no       | See below     | The ID of this SIP endpoint in the format `sip:<username>@<addr>`             |
| `username`       | `string`                          | no       | none          | The username to register with the SIP provider                                |
| `password`       | `string`                          | no       | none          | The password to register with the SIP provider                                |
| `realm`          | `string`                          | no       | none          | The realm of the given username/password pair                                 |
| `registrar`      | `string`                          | no       | none          | The SIP URI of the registrar in the format `sip:<domain>`                     |
| `outbound_proxy` | `string`                          | no       | none          | The SIP proxy to send all requests to in the format `sip:<domain>`            |
| `nat_ping_delta` | `string`                          | no       | 30 seconds    | Seconds between ping and pong to keep the NAT binding alive                   |
| `stun_server`    | `string`                          | no       | none          | The host and optional port of the STUN server in the format `<host>[:<port>]` |
| `enforce_qop`    | `bool`                            | no       | `false`       | `true` to enforce quality of protection on SIP authentication                 |
| `rtp_port_range` | [RTP port range](#rtp-port-range) | no       | 40000-49999   | The port range for the SIP RTP/RTCP connections                               |

If `ìd` is not set, it is generated as `sip:<username>@<addr>` where `<addr>` may be replaced by the public address discovered using the STUN server.

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
username = "user"
password = "pass"
realm = "asterisk"
registrar = "sip:sip.example.org"
stun_server = "stun.example.org:3478"

[sip.rtp_port_range]
start = 40000
end = 49999
```
