---
title: Controller
---

# Administration Guide for the {{ product_name }} Controller

## General information about the service

- [Configuration](./core/configuration.md)
- [HTTP Server](./core/http_server.md) on which the controller offers its service
- [Migration Guide for Updating to New Versions](./migration/README.md)
- [Command-Line Usage of the Controller](./cli/README.md)
- [Configuration of Multiple Tenants](./advanced/tenants.md)
- [Configuration of Different Tariffs and Their Capabilities](./advanced/tariffs.md)
- [Execution of Maintenance Jobs](./cli/jobs.md)
- [Modules That can be Used in Meetings](./advanced/modules.md)
- [Access Control](./advanced/acl.md)
- [Call-In](./advanced/call_in.md)
- [Default and Fallback Values](./advanced/defaults.md)
- [Endpoints](./core/endpoints.md)
- [Logging](./core/logging/log_output.md)
- [Metrics](./core/logging/metrics.md)
- [Personal Data Storage](./personal_data_storage.md)

## Interaction between {{ product_name }} Controller and other services

### Services required by {{ product_name }} Controller

- [Database](./core/database.md)
- [Keycloak](./core/keycloak.md)
- [MinIO](./core/minio.md)

### Services required by {{ product_name }} Controller for certain setups

- [RabbitMQ](./core/rabbitmq.md) for communication with [services that interact with {{ product_name }} Controller](#services-that-can-interact-with-the-controller)
- [Redis](./core/redis.md) for the clustered mode of {{ product_name }} Controller

### Services that the Controller can be integrated with

- [Shared Folders on External Systems](./advanced/additional_services/shared_folder.md)
- [Tracing](./core/logging/tracing.md)
- [Etherpad](./advanced/additional_services/etherpad.md)
- [SpaceDeck](./advanced/additional_services/spacedeck.md)

### Services that can interact with the Controller

- [{{ product_name }} Obelisk](./advanced/additional_services/obelisk.md) for handling dial-in from telephone line
- [{{ product_name }} Recorder](./advanced/additional_services/recorder.md) for recording meetings
- [{{ product_name }} SMTP-Mailer](./advanced/additional_services/smtp_mailer.md) for sending E-Mail notifications to users

## Under the hood

- [OIDC Authentication Flow](./under_the_hood/oidc_auth.md)
- [Handling of WebAPI Requests](./under_the_hood/http_requests.md)
- [Participant Lifecycle and States](./under_the_hood/participant_states.md)
