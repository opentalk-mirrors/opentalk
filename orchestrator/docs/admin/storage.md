# Storage

The {{ product_name }} Orchestrator keeps track of the registered service instances, their load and the resources
(e.g. rooms) assigned to them. This state is held in a storage backend.

## Configuration

The section in the [configuration file](README.md) is called `storage`. The section is required.

| Field  | Type     | Required         | Default value | Description                                        |
| ------ | -------- | ---------------- | ------------- | -------------------------------------------------- |
| `kind` | `enum`   | yes              | -             | The storage backend, either `local` or `redis`     |
| `url`  | `string` | for `kind=redis` | -             | The URL of the Redis instance                      |

### Local storage

With `kind = "local"` the state is kept in the memory of the orchestrator process. The state is lost when the
orchestrator restarts and cannot be shared with other orchestrators. Use this backend only for deployments with a
single orchestrator.

```toml
[storage]
kind = "local"
```

### Redis storage

With `kind = "redis"` the state is kept in Redis and can be shared by multiple orchestrators. This is required when
more than one orchestrator is deployed.

```toml
[storage]
kind = "redis"
url = "redis://localhost:6379"
```

The URL follows the usual Redis URL format, e.g. `redis://:<password>@<host>:<port>/<db>`.

Requirements for the Redis instance:

- Redis 7.2 or newer. The orchestrator always connects with the RESP3 protocol.
- Keyspace notifications for keyspace events (`K`) and expired events (`x`). The orchestrator reads the
  `notify-keyspace-events` setting on startup and enables missing flags with `CONFIG SET`. If the `CONFIG` command
  isn't permitted for the orchestrator, e.g. on managed Redis offerings, enable the flags (`notify-keyspace-events Kx`)
  in the Redis configuration beforehand.
