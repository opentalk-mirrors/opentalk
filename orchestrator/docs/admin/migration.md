---
title: Migration guide
---

# Migration guide

This guide describes changes to the configuration file and how to update an
existing setup between releases.

## Migrating from v0.12 to v0.13

### New optional OpenTelemetry tracing

The orchestrator can now export spans to an
[OTLP](https://opentelemetry.io/docs/specs/otlp/) endpoint. The exporter is
configured with new optional fields in the existing `[logging]` section. The
section and its `directives` field are unchanged, so existing configurations
keep working without modification.

To enable trace export, set the endpoint (and optionally the service metadata):

```toml
[logging]
#directives = ["warn", "opentalk_orchestrator=info"]

# Export spans to an OTLP endpoint. When unset, trace export stays disabled.
otlp_tracing_endpoint = "http://localhost:4317"
#service_name = "orchestrator"
#service_namespace = "opentalk"
#service_instance_id = "627cc493-f310-47de-96bd-71410b7dec09"
```

No action is required unless you want to enable OTLP export.
