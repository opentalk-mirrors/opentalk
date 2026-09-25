---
title: Orchestrator
---

# Administration guide for the OpenTalk Orchestrator

## General information about the service

- [Migration Guide](./migration.md)

### Environment

- `ORCHESTRATOR_LOG` to configure the log filter. It takes precedence over
  `RUST_LOG`. When neither is set, the default `warn,opentalk_orchestrator=info`
  is used.
