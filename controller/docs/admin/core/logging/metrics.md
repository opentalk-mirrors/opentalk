# Metrics

The {{ product_name }} controller is collecting metrics and is exposing them through the `/metrics` endpoint.

## Configuration

The section in the [configuration file](../configuration.md) is called `metrics`.

By default, the `/metrics` endpoint refuses all connections. The access can be configured with an allowlist.

| Field       | Type     | Required | Default value | Description                                                       |
| ----------- | -------- | -------- | ------------- | ----------------------------------------------------------------- |
| `allowlist` | `string` | no       | -             | List of IP-Addresses or Subnet which are allowed to fetch metrics |

### Examples

#### Access denied (default)

```toml
[metrics]
allowlist = []
```

#### Only allow localhost

```toml
[metrics]
allowlist = ["127.0.0.0/8", "::ffff:0:0/96"]
```

#### Allow IPs and subnets

```toml
[metrics]
allowlist = ["1.1.1.1", "127.0.0.0/8"]
```

## Web-API

The metrics can be accessed via the `/metrics` endpoint in the [OpenMetrics Text Format](https://github.com/OpenObservability/OpenMetrics), which is utilized by [prometheus](https://prometheus.io/docs/instrumenting/exposition_formats/#openmetrics-text-format).

All metrics listed below carry an `otel_scope_name` label: `ot-controller` for metrics originating in the controller itself, `ot-roomserver` for signaling metrics coming from the embedded roomserver.
The OpenTelemetry Prometheus exporter additionally emits a `target_info` gauge carrying SDK and service metadata (`service_name`, `telemetry_sdk_language`, `telemetry_sdk_name`, `telemetry_sdk_version`).

### HTTP endpoint metrics

| Key                            | Type      | Labels                        | Description                                              |
| ------------------------------ | --------- | ----------------------------- | -------------------------------------------------------- |
| `web_request_duration_seconds` | histogram | `method`, `handler`, `status` | Duration of HTTP requests handled by the controller      |
| `web_response_sizes_bytes`     | histogram | `method`, `handler`, `status` | Body size of HTTP responses with a known length          |
| `web_issued_email_tasks_count` | counter   | `mail_task_kind`              | Number of outbound email tasks the controller has queued |

### Database metrics

| Key                           | Type      | Labels  | Description                                   |
| ----------------------------- | --------- | ------- | --------------------------------------------- |
| `sql_execution_time_seconds`  | histogram |         | Time taken to execute a single SQL query      |
| `sql_errors_total`            | counter   | `error` | Number of SQL queries that returned an error  |
| `sql_dbpool_connections`      | histogram |         | Number of currently non-idling db connections |
| `sql_dbpool_connections_idle` | histogram |         | Number of currently idling db connections     |

### Redis metrics

Only present when [Redis](../redis.md) is configured.

| Key                                    | Type      | Labels    | Description                  |
| -------------------------------------- | --------- | --------- | ---------------------------- |
| `redis_command_execution_time_seconds` | histogram | `command` | Redis command execution time |

### Signaling metrics

Provided by the roomserver. Only present on the controller's `/metrics` endpoint when `roomserver.kind = "internal"` (see [RoomServer](../roomserver.md)); with `roomserver.kind = "external"`, scrape the roomserver's own `/metrics` endpoint instead.

| Key                                        | Type      | Labels               | Description                                                                                            |
| ------------------------------------------ | --------- | -------------------- | ------------------------------------------------------------------------------------------------------ |
| `signaling_created_rooms_count`            | counter   |                      | Number of created rooms                                                                                |
| `signaling_destroyed_rooms_count`          | counter   |                      | Number of destroyed rooms                                                                              |
| `signaling_created_breakout_rooms_count`   | counter   |                      | Number of breakout rooms that have been opened                                                         |
| `signaling_destroyed_breakout_rooms_count` | counter   |                      | Number of breakout rooms that have been closed                                                         |
| `signaling_connection_count`               | gauge     | `participation_kind` | Number of connections by kind (user, guest, recorder, transcription, call_in, registered_call_in)      |
| `signaling_connections_per_room`           | gauge     | `bucket`             | Connections per room by bucket (2, 10, 25, 50, 100, 200, 300)                                          |
| `signaling_room_life_time`                 | histogram |                      | Time rooms were active                                                                                 |
| `signaling_connection_meeting_time`        | histogram |                      | Time a connection was connected to a meeting room                                                      |
| `signaling_congested_connections`          | counter   |                      | Number of connections the server dropped because they could not keep up with the outgoing message rate |
