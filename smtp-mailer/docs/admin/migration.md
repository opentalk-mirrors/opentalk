---
title: Migration guide
---

# Migration guide

## Migrating from v0.19 to v0.20

The SMTP Mailer now shares its tracing and logging setup with the other
{{ product_name }} services.

### Log output format changed

Log formatting switched from `env_logger` to the `tracing` subscriber used by
the other services. The directives you set via `RUST_LOG` keeps working (but the
new `SMTP_MAILER_LOG` takes precedence), but the format of the emitted log lines
changed. Adjust any log parsing or alerting that depends on the previous format.

### `html5ever` and `selectors` are no longer capped automatically

The log filter is now taken from a single source; the sources are no longer
merged. The built-in defaults still cap `html5ever` and `selectors` at `warn`,
but when you set `RUST_LOG` it replaces the defaults entirely. If you raise the
global level and want to keep those crates quiet, add the caps explicitly:

```sh
RUST_LOG=debug,html5ever=warn,selectors=warn opentalk-smtp-mailer
```

### New optional `[tracing]` section

You can now export spans to an [OTLP](https://opentelemetry.io/docs/specs/otlp/)
endpoint and set default log directives in the configuration file. The section
is optional; if you omit it, logging behaves as before. See
[Log Output](log_output.md) for details.

```toml
[tracing]
otlp_tracing_endpoint = "http://localhost:4317"
default_directives = ["warn", "html5ever=warn", "selectors=warn"]
```

No action is required unless you want to enable OTLP export or rely on the exact
previous log line format or crate-capping behavior.

## Migrating from v0.18 to v0.19

### `guest_link_builder` configuration option removed

Invite links have been replaced by join links, and the `invite_code` is no
longer used. As part of this change, the `guest_link_builder` option in the
`[frontend]` section has been removed.

If your configuration still sets `guest_link_builder`, the service keeps
starting but logs an `Unknown configuration key` warning. Remove the option from
your configuration to silence the warning:

```toml
[frontend]
#guest_link_builder = "{base_url}/invite/{invite_id}"
```

## Migrating from v0.17 to v0.18

### Environment variable prefix changed to `OPENTALK_MAIL`

The prefix used for configuration environment variables changed from `MAILER` to
`OPENTALK_MAIL`, aligning the {{ product_name }} SMTP Mailer with the other
{{ product_name }} services. Nested fields are still separated by two
underscores `__`.

For example, the queue name is now set like this:

```sh
OPENTALK_MAIL_RABBIT_MQ__QUEUE_NAME=opentalk_mailer
```

The old `MAILER` prefix is still supported for backwards compatibility, so
existing deployments keep working without changes. It is deprecated and should
be replaced with the new prefix. If a setting is provided under both prefixes,
the value from `OPENTALK_MAIL` takes precedence.

### RabbitMQ queue is now durable and non-exclusive

The queue consumed by the SMTP Mailer is now declared as durable and
non-exclusive so that tasks survive a broker restart.

If a queue with the configured name already exists on your broker with different
properties (for example a non-durable queue from a previous version), RabbitMQ
rejects the new declaration with a `PRECONDITION_FAILED` error and the service
fails to start. In that case, delete the existing queue so it can be recreated
with the new properties.
