---
sidebar_position: 1
---

# Configuring {{ product_name }} SMTP Mailer

When the SMTP mailer gets started, it loads the configuration from the
environment. It reads the settings in this order:

- Read environment variables which have a specific name, see section
  [Environment variables](#environment-variables).
- Load from a configuration file which defaults to `config.toml` in the current
  working directory.

## Sections in the configuration file

Functionality that can be configured through the configuration file:

- [RabbitMQ](rabbitmq.md)
- [Frontend](frontend.md)
- [Languages](languages.md)
- [SMTP server](smtp.md)
- [Support contact](support_contact.md)
- [Template builder](template_builder.md)
- [Templates](templates.md)

## Environment variables

Settings in the configuration file can be overwritten by environment variables,
nested fields are separated by two underscores `__`. The pattern looks like
this:

```sh
OPENTALK_MAIL_<field>__<nested-field>…
```

### Limitations

Some settings can not be overwritten by environment variables. This is for
example the case for entries in lists, because there is no environment variable
naming pattern that could identify the index of the entry inside the list.

### Examples

In order to set the `rabbit_mq.mail_task_queue` field, this environment variable could be used:

```sh
OPENTALK_MAIL_RABBIT_MQ__MAIL_TASK_QUEUE=opentalk_mailer
```

## Example configuration file

This file can be found in the source code distribution under `example/smtp-mailer.toml`

<!-- begin:fromfile:config/smtp-mailer.toml.md -->

```toml
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

[rabbit_mq]
# The URL to use to connect to the rabbit mq broker
url = "amqp://username:password@localhost/%2F"
# The queue to subscribe to
queue_name = "opentalk_mailer"

# The number of seconds after which the processing of a single mailer task will be aborted.
# In that case, the task will be rejected to RabbitMQ with the requeue flag on, so RabbitMQ
# will attempt to deliver it again.
#
# This should be strictly shorter than the RabbitMQ delivery acknowledge timeout
# which can be set in the RabbitMQ configuration and defaults to 30 minutes
# (1800 seconds).
#
# When set to 0, no timeout exists and the task will block infinitely if it
# doesn't finish on its own. This is the same behavior that was implemented in
# SMTP-Mailer 0.14.0 and earlier.
#task_processing_timeout_seconds = 1200

[smtp]
# SMTP Cleartext: smtp://user:pass@mailserver.example.org:1234?disable_starttls=true
# SMTP with StartTLS: smtp://user:pass@mailserver.example.org:1234
# SMTP with implicit TLS: smtps://user:pass@mailserver.example.org:1234
# Note: Encode the URL properly:
#       - Significant characters (e.g. the '@' separating the user credentials from the domain) must be written literally.
#       - Special characters within the URL parts must be paraphrased (e.g. '@' replaced with '%40')
#       Example: smtps://user%40maildomain.example.org:pass@mailserver.example.org
#smtp_server = "smtp://localhost:1025?disable_starttls=true"
# Set the From email address according to the requirements of your SMTP server.
#from_name = "OpenTalk"
#from_email = "no-reply@example.org"

[template_builder]
# Link builder templates. `base_url` and `x_id` will be replaced. Need when the frontend uses different paths.
#join_link_builder = "{base_url}/room/{room_id}"
#dashboard_event_link_builder = "{base_url}/dashboard/meetings/{event_id}"

[frontend]
#base_url = "https://opentalk.example.org"
#data_protection_url = "https://opentalk.example.org/dataprotection"

[templates]
# Path to templates. These are loaded at runtime and are not compiled in.
#registered_invite = {html = "resources/templates/registered_invite.html", txt = "resources/templates/registered_invite.txt"}
#unregistered_invite = {html = "resources/templates/unregistered_invite.html", txt = "resources/templates/unregistered_invite.txt"}
#external_invite = {html = "resources/templates/external_invite.html", txt = "resources/templates/external_invite.txt"}
#registered_event_update = {html = "resources/templates/registered_event_update.html", txt = "resources/templates/registered_event_update.txt"}
#unregistered_event_update  = {html = "resources/templates/unregistered_event_update.html", txt = "resources/templates/unregistered_event_update.txt"}
#external_event_update = {html = "resources/templates/external_event_update.html", txt = "resources/templates/external_event_update.txt"}
#registered_event_cancellation = {html = "resources/templates/registered_event_cancellation.html", txt = "resources/templates/registered_event_cancellation.txt"}
#unregistered_event_cancellation = {html = "resources/templates/unregistered_event_cancellation.html", txt = "resources/templates/unregistered_event_cancellation.txt"}
#external_event_cancellation = {html = "resources/templates/external_event_cancellation.html", txt = "resources/templates/external_event_cancellation.txt"}
#registered_uninvite = {html = "resources/templates/registered_uninvite.html", txt = "resources/templates/registered_uninvite.txt"}
#unregistered_uninvite = {html = "resources/templates/unregistered_uninvite.html", txt = "resources/templates/unregistered_uninvite.txt"}
#external_uninvite = {html = "resources/templates/external_uninvite.html", txt = "resources/templates/external_uninvite.txt"}

[languages]
#default_language = "de-DE"

[monitoring]
addr = "0.0.0.0"
port = 11411

# Optional OpenTelemetry tracing configuration.
#[tracing]
#otlp_tracing_endpoint = "http://localhost:4317"
#service_name = "smtp-mailer"
#service_namespace = "opentalk"
#service_instance_id = "627cc493-f310-47de-96bd-71410b7dec09"
# The `SMTP_MAILER_LOG` and `RUST_LOG` environment variables take precedence over this setting.
#default_directives = ["warn", "html5ever=warn", "selectors=warn"]

# Optional support contact information
# Adds a support contact section to the template
[support_contact]
# Support phone contact
phone = "+49123321123"
# Support mail contact
mail = "support@example.com"
```

<!-- end:fromfile:config/smtp-mailer.toml.md -->
