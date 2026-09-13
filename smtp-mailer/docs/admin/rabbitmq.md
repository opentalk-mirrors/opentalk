---
sidebar_position: 2
---

# RabbitMQ

When started up, the {{ product_name }} SMTP Mailer connects to the configured
[RabbitMQ](https://www.rabbitmq.com/) queue and consumes the messages it
receives from there.

## Configuration

The section in the [configuration file](configuration.md) is called `rabbit_mq`.

| Field                             | Type     | Required | Default value                       | Description                                                                                                                                     |
| --------------------------------- | -------- | -------- | ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `url`                             | `string` | no       | "amqp://guest:guest@localhost:5672" | The RabbitMQ broker URL                                                                                                                         |
| `queue_name`                      | `string` | yes      | -                                   | The name of the RabbitMQ queue for the SMTP mailer                                                                                              |
| `task_processing_timeout_seconds` | `uint`   | no       | 1200 (equal to 20 minutes)          | The timeout after which the sending task must be finished, otherwise it will be returned to RabbitMQ with the requeue flag on. 0 means infinite |

### Example

```toml
[rabbit_mq]
url = "amqp://username:password@localhost/%2F"
mail_task_queue = "opentalk_mailer"
task_processing_timeout_seconds = 1200
```
