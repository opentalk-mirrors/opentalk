# SMTP Mailer

The [{{ product_name }} SMTP-Mailer](https://gitlab.opencode.de/opentalk/smtp-mailer) is a service whose main purpose is to send out E-mail invites, updates and cancellations of meetings.

## Deploy SMTP-Mailer

The SMTP-Mailer has to be configured and started separately.

You can clone the repository and build the Docker image yourself or download the image directly from the [Container Registry](https://gitlab.opencode.de/opentalk/smtp-mailer/container_registry).

Configure the SMTP-Mailer, you can refer to its [example config](https://gitlab.opencode.de/opentalk/smtp-mailer/-/blob/main/config.toml.example?ref_type=heads).
Note that the Controller and Mailer have to use the same rabbitmq service.

## Configuration

The SMTP-Mailer configuration is part of the [`rabbitmq`](../../core/rabbitmq.md) section in the [configuration file](../../core/configuration.md).

| Field             | Type     | Required | Default value | Description                                                                                                                           |
| ----------------- | -------- | -------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| `mail_task_queue` | `string` | no       | -             | The rabbitmq queue name for the SMTP-Mailer. The SMTP-Mailer configuration  has the same field and needs to have the same queue name. |

### Examples

#### Default setup

```toml
[rabbit_mq]
url = "amqp://username:password@host/%2F"
mail_task_queue = "opentalk_mailer"
```
