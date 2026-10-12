---
title: Orchestrator
---

# Administration guide for the {{ product_name }} Orchestrator

The {{ product_name }} Orchestrator manages scalable services in an {{ product_name }} deployment. It is somewhat like
a stateful load balancer. It proxies the internal web and signaling APIs of orchestrated services and forwards requests
to the appropriate service instance. By doing so, the orchestrator allows deployments to have multiple Roomservers,
Recorders or Transcription instances.

Not all services need to be scaled at once. It's possible to have a deployment where only the one of the services is
scaled through the orchestrator.

## General information about the service

- [Configuration](./configuration.md)
- [Deployment](./deployment/README.md)
- [Migration Guide](./migration.md)

## Interaction between {{ product_name }} Orchestrator and other services

- [Controller](./deployment/controller.md)
- [RoomServer](./deployment/roomserver.md)
- [Recorder](./deployment/recorder.md)
- [Transcription](./deployment/transcription.md)

### Services required by {{ product_name }} Orchestrator for certain setups

- [Redis](./storage.md#redis-storage) when running multiple Orchestrator instances
