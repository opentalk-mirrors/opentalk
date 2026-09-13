# HTTP Server

The {{ product_name }} Controller provides its service to clients through a built-in HTTP
server.

Services provided:

- [`v1` REST API](../../developer/api.md) under `/v1`
- [Signaling](../../developer/signaling/README.md) for meetings under `/signaling`
- [Metrics](./logging/metrics.md) under `/metrics`

## Configuration

The section in the [configuration file](./configuration.md) is called `http`.

| Field               | Type                                      | Required | Default value | Description                                                                                    |
| ------------------- | ----------------------------------------- | -------- | ------------- | ---------------------------------------------------------------------------------------------- |
| `addr`              | `string`                                  | no       | -             | IP address or hostname to which to listen for incoming connections                             |
| `port`              | `uint`                                    | no       | `11311`       | TCP port number where the HTTP server can be reached                                           |
| `upload_size_limit` | `usize`                                   | no       | `100,000,000` | The maximum size for an uploaded asset, in bytes                                               |
| `tls`               | [TLS configuration](#tls-configuration)   | no       | -             | When present, the HTTP server will use TLS, when absent it will serve under a plain connection |
| `cors`              | [CORS configuration](#cors-configuration) | no       | -             | Configure the CORS headers                                                                     |

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

### TLS configuration

| Field         | Type     | Required | Default value | Description                                                                 |
| ------------- | -------- | -------- | ------------- | --------------------------------------------------------------------------- |
| `certificate` | `string` | yes      | -             | Path to the file containing the TLS certificate in DER-encoded x.509 format |
| `private_key` | `string` | yes      | -             | Path to the file containing the private TLS key in pkcs8 format             |

### Examples

#### Plain HTTP on all addresses

```toml
[http]
port = 80
```

#### Plain HTTP on localhost only (IPv4 and IPv6 if available)

```toml
[http]
addr = "localhost"
port = 80
```

#### Plain HTTP on an IPv4 address

```toml
[http]
addr = "192.0.2.0"
port = 80
```

#### HTTP over TLS

```toml
[http]
port = 443

[http.tls]
certificate = "/etc/ssl/certs/example.org.pem"
private_key = "/etc/ssl/keys/example.org.key"
```

#### HTTP over TLS on an IPv6 address

```toml
[http]
addr = "2001:0DB8::1337:DEAD:CAFE"
port = 443

[http.tls]
certificate = "/etc/ssl/certs/example.org.pem"
private_key = "/etc/ssl/keys/example.org.key"
```

### CORS configuration

| Field            | Type       | Required | Default value | Description                                                              |
| ---------------- | ---------- | -------- | ------------- | ------------------------------------------------------------------------ |
| `allowed_origin` | `string[]` | no       | -             | A list of allowed origins, either origin URLs, or one single `"*"` entry |

#### Default behavior

By default, the `Access-Control-Allow-Origin` header is derived from the
`base_url` field in the [Frontend configuration](frontend.md). All elements
that are not needed when used as the CORS origin (username, password, path,
query and fragment parts) will be stripped for usage as a CORS header. If
`base_url` has a value of `"https://example.com/opentalk/"`, then the derived
value for the origin is `"https://example.com"`.

This default configuration should be suitable for most standard deployments, so
the `allowed_origin` configuration is not required there.

#### Example configurations

##### Wildcard

This allows browsers to access the controller API from websites regardless their
origin.

If the list contains a `"*"` entry, no other entries can be present, otherwise
this is considered a configuration error which prevents the controller from
starting.

```toml
[http.cors]
allowed_origin = ["*"]
```

##### Specific origins

When configuring the allowed origins like this, one entry corresponding to
`frontend.base_url` should be included in the list as well, otherwise a frontend
deployed there won't be able to access the controller.

```toml
[http.cors]
allowed_origin = ["https://example.com", "https://opentalk.example.com:1337"]
```
