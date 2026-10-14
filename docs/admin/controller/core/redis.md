# Session Data (Redis)

The {{ product_name }} controller can be run in *standalone* mode, where a single
`opentalk-controller` provides the web api and signaling service, or it can run
in a *clustered* mode, where multiple `opentalk-controller` nodes provide
the service in parallel and synchronize through [Redis](https://redis.com/)
for caching data related to a video conference session.

Up until version `v0.14` of the `opentalk-controller`, the redis service was
mandatory, starting with `v0.15` it is possible to leave the whole configuration
section out to operate the controller in *standalone* mode.

## Configuration

The section in the [configuration file](./configuration.md) is called `redis`.

| Field | Type     | Required | Default value             | Description                                           |
| ----- | -------- | -------- | ------------------------- | ----------------------------------------------------- |
| `url` | `string` | no       | "redis://localhost:6379/" | TCP port number where the Redis server can be reached |

### Examples

#### Default setup

The default setup since `0.15` is to leave the `[redis]` section out entirely,
therefore operating the `opentalk-controller` in *standalone* mode.

```toml
# [redis]
#url = "redis://localhost:6379/"
```

#### Default Redis URL

Using the default redis url requires the `[redis]` section to be present. If the
`url` field is absent, then the default redis URL will be used. This was the
default configuration up until `v0.14` of the {{ product_name }} controller.

```toml
[redis]
#url = "redis://localhost:6379/"
```

#### Custom Redis URL

A custom Redis URL can be specified as following:

```toml
[redis]
url = "redis://localhost:6399/"
```
