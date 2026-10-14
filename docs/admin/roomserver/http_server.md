# HTTP Server

The {{ product_name }} RoomServer provides its service to clients through a built-in HTTP server.

Services provided:

- `v1` REST API under `/v1`
- [Metrics](./observability/metrics.md) on the port specified in the metrics settings under `/metrics`
- [Monitoring](./observability/monitoring.md) on the port specified in the monitoring settings.
- [OpenAPI](https://www.openapis.org/what-is-openapi) under `/docs/openapi.json`
- [Swagger](https://swagger.io/) REST API documentation under `/swagger`

## Configuration

The section in the [configuration file](./configuration.md) is called `http`.

| Field            | Type                 | Required | Default value | Description                                                        |
| ---------------- | -------------------- | -------- | ------------- | ------------------------------------------------------------------ |
| `addr`           | `string`             | no       | -             | IP address or hostname to which to listen for incoming connections |
| `port`           | `uint`               | no       | `11333`       | TCP port number where the REST API can be reached                  |
| `service_url`    | `string`             | no       | -             | The URL of the roomserver that is reachable by internal services   |
| `public_url`     | `string`             | yes      | depends       | The publicly reachable URL of this server                          |
| `api_keys`       | Key/value pair array | yes      | -             | The api keys for internal service endpoints                        |
| `enable_openapi` | `bool`               | no       | `false`       | Enable the OpenAPI and the corresponding Swagger endpoints         |

### Listening address

By default, the service will accept requests on both the IPv4 and IPv6
interfaces if either a hostname is set for `addr`, or if no `addr` value is set
at all.

The exception to this rule is `"::0"` which will bind to both the IPv4
`UNSPECIFIED` address and the IPv6 `UNSPECIFIED` address at the same time,
accepting requests on any address for both protocols. If the operating system
provides no IPv6 support, or the service should not bind to an IPv6 interface,
`"0.0.0.0"` can be used instead, which will only bind to the IPv4 `UNSPECIFIED`
address.

A hostname or fully qualified domain name will bind to whatever the name
resolution returns, either one or both IP protocols.

An explicit IPv4 or IPv6 address, will bind exactly to the corresponding IP protocol.

### Service URL

The `service_url` value is what internal services (e.g., the recorder) use to reach the RoomServer instance.
When this value is not provided, the RoomServer tries to use the value of the `public_url`.
If the `public_url` isn't provided either it falls back to an unspecified IPv4 address (`0.0.0.0`).

### Public URL

The `public_url` value is what external services use to reach the RoomServer instance that hosts their target room.
In multi-instance setups the controller forwards this value to the clients.
It is included in a successful response on the `/rooms/{room_id}/token` endpoint.

### API Keys

The RoomServer can have multiple API keys configured that allow external
services to access its REST API. An API key can be configured as string
(`"<key_id>:<key_secret>"`) or as key/value pair (`{id = "<key_id>", secret = "<key_secret>"}`).

### Enable OpenAPI

When set to `true`, enables the OpenAPI endpoint under `/docs/openapi.json` and
the corresponding Swagger endpoint under `/swagger`. These endpoints are useful
for development, but have a heavy negative impact on the performance of all REST API
requests. It is recommended to disable this in production.
