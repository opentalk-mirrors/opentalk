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

## Trace context propagation

The Orchestrator uses the
[W3C Trace Context](https://www.w3.org/TR/trace-context-1) `traceparent` and
`tracestate` HTTP headers. When OTLP tracing export is enabled, valid context
received on an HTTP request becomes the parent of the Orchestrator request span,
and internal HTTP requests to other services carry the active context. When OTLP
tracing is disabled (the default), these headers are neither parsed nor emitted.

Configure all {{ product_name }} services to export to the same collector or
tracing backend to display their spans as one trace in tools such as Jaeger.
Propagation currently covers HTTP traffic between these services. Services
without OTLP tracing export terminate the visible trace, even when they receive
a `traceparent` header.

!!! warning "Treat incoming trace headers as untrusted"

    When tracing is enabled, `traceparent` and `tracestate` are accepted from any
    client. To avoid spoofed or malformed trace context, strip or overwrite these
    headers at a trusted reverse proxy for requests from untrusted sources, as
    recommended by the [OpenTelemetry security best practices](https://opentelemetry.io/docs/concepts/context-propagation/#security-best-practices).
