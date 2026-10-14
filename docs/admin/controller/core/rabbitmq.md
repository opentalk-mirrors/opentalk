# Message Queue (RabbitMQ)

RabbitMQ is required if you want to use [services that interact with {{ product_name }} Controller](../README.md#services-that-can-interact-with-the-controller).

## Configuration

The section in the [configuration file](./configuration.md) is called `rabbit_mq`.

| Field                  | Type     | Required | Default value                       | Description                                                                                          |
| ---------------------- | -------- | -------- | ----------------------------------- | ---------------------------------------------------------------------------------------------------- |
| `url`                  | `string` | no       | "amqp://guest:guest@localhost:5672" | The RabbitMQ broker URL connection                                                                   |
| `mail_task_queue`      | `string` | no       | -                                   | Name of the RabbitMQ queue for the [SMTP-Mailer](../advanced/additional_services/smtp_mailer.md)     |
| `recording_task_queue` | `string` | no       | -                                   | Name of the RabbitMQ queue for the [recorder](../advanced/additional_services/recorder.md)           |
| `min_connections`      | `uint`   | no       | 10                                  | Minimum connections to retain when removing stale connections                                        |
| `max_channels`         | `uint`   | no       | 100                                 | Maxmimum number of AMQP channels per connection allowed                                              |
| `message_ttl_seconds`  | `uint`   | no       | 3600                                | The number of seconds after which RabbitMQ will discard messages that haven't been processed by then |

### Examples

#### With mail worker queue

```toml
[rabbit_mq]
mail_task_queue = "opentalk_mailer"
```

#### With all configurations

```toml
[rabbit_mq]
url = "amqp://username:password@host/%2F"
mail_task_queue = "opentalk_mailer"
recording_task_queue = "opentalk_recorder"
min_connections = 10
max_channels_per_connection = 100
message_ttl_seconds = 3600
```
