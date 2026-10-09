# Migration Guide

This guide describes changes to the configuration file and logging behavior and
how to update an existing setup between releases.

## `v0.4` -> `v0.5`

The Transcription service now shares its tracing and logging setup with the
other OpenTalk services.

### Log output format changed

Log formatting switched from `env_logger` to the `tracing` subscriber used by
the other services. The directives you set via `TRANSCRIPTION_LOG` or `RUST_LOG`
keep working unchanged, but the format of the emitted log lines changed. Adjust
any log parsing or alerting that depends on the previous format.

### New optional `[tracing]` section

You can now export spans to an [OTLP](https://opentelemetry.io/docs/specs/otlp/)
endpoint and set default log directives in the configuration file. The section
is optional; if you omit it, logging behaves as before. See the
[Tracing configuration](configuration/tracing.md) for details.

```toml
[tracing]
otlp_tracing_endpoint = "http://localhost:4317"
default_directives = ["warn", "opentalk_transcription=info"]
```

No action is required unless you want to enable OTLP export or rely on the exact
previous log line format.
