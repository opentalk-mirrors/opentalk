# Metrics

The {{ product_name }} RoomServer is collecting metrics and can expose them through the `/metrics` endpoint. The metrics can be accessed via the `/metrics` endpoint in the [OpenMetrics Text Format](https://github.com/OpenObservability/OpenMetrics), which is utilized by [prometheus](https://prometheus.io/docs/instrumenting/exposition_formats/#openmetrics-text-format).

## Configuration

The section in the [configuration file](../configuration.md) is called `metrics`.

By default the `/metrics` endpoint is disabled. It can be enabled, by adding the `[metrics]` section to the configuration file. By default, the `/metrics` endpoint refuses all connections. The access can be configured with an allowlist.

| Field       | Type      | Required | Default value | Description                                                       |
| ----------- | --------- | -------- | ------------- | ----------------------------------------------------------------- |
| `port`      | `integer` | no       | 11412         | Port of the metrics endpoint                                      |
| `allowlist` | `string`  | no       | -             | List of IP-Addresses or Subnet which are allowed to fetch metrics |

## Metrics exposed

| Key                                      | Type      | Labels                   | Description                                                                           |
| ---------------------------------------- | --------- | ------------------------ | ------------------------------------------------------------------------------------- |
| api_http_requests_total                  | counter   | method, endpoint, status | Number of HTTP requests handled                                                       |
| api_http_requests_pending                | gauge     | method, endpoint         | Number of currently in-flight http requests                                           |
| api_http_requests_duration_seconds       | histogram | method, endpoint, status | Request duration for all HTTP requests handled                                        |
| api_http_response_body_size              | histogram | method, endpoint         | Response body sizes for all handled HTTP requests                                     |
| signaling_created_rooms_count            | counter   |                          | Number of created rooms                                                               |
| signaling_destroyed_rooms_count          | counter   |                          | Number of destroyed rooms                                                             |
| signaling_created_breakout_rooms_count   | counter   |                          | Number of created breakout rooms                                                      |
| signaling_destroyed_breakout_rooms_count | counter   |                          | Number of destroyed breakout rooms                                                    |
| signaling_connection_count               | gauge     | participation_kind       | Number of connections by kind (user, guest, recorder, call_in, registered_call_in)    |
| signaling_connections_per_room           | gauge     | bucket                   | Connections per room by bucket (2, 10, 25, 50, 100, 200, 300)                         |
| signaling_connection_meeting_time        | histogram |                          | Time a connection was connected to a meeting room                                     |
| signaling_room_life_time                 | histogram |                          | Time rooms were active                                                                |
| signaling_congested_connections          | counter   |                          | Number of connections the server dropped because the outgoing message buffer was full |
