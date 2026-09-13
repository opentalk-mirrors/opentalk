# Monitoring

The {{ product_name }} RoomServer provides a simple built-in HTTP service for monitoring purposes. This HTTP service is only provided when configured. When enabled the server returns `UP` when the RoomServer is running, but not yet ready to accept requests and `READY` once the RoomServer is ready to accept requests.

## Configuration

The section in the [configuration file](../configuration.md) is called `monitoring`.

| Field  | Type     | Required | Default value | Description                            |
| ------ | -------- | -------- | ------------- | -------------------------------------- |
| `port` | `int`    | no       | 11411         | The port for the monitoring server.    |
| `addr` | `string` | no       | 0.0.0.0       | The address for the monitoring server. |
