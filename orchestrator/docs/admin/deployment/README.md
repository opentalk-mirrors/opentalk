# Deployment

A deployment with the {{ product_name }} Orchestrator requires configuration changes in the {{ product_name }}
Controller and in the services that should be scaled.

Scaling a service with the orchestrator takes three steps:

1. Add the `orchestrator` section to the configuration of the proxied service.
2. In the `controller.toml`, replace the `url` and `api_key` of the proxied service with the orchestrator's proxy URL
   and API key.
3. In the `orchestrator.toml`, add the API keys of each proxied service to the `services.keys` configuration.

The following services can be scaled when configured with the orchestrator:

| Service                          | Orchestrator endpoint |
| -------------------------------- | --------------------- |
| {{ product_name }} RoomServer    | `/roomserver/`        |
| {{ product_name }} Recorder      | `/recording/`         |
| {{ product_name }} Transcription | `/transcription/`     |

## Related component configuration

- [Controller](controller.md)
- [Recorder](recorder.md)
- [Roomserver](roomserver.md)
- [Transcription](transcription.md)

## Running multiple Orchestrators

The orchestrator can run with multiple instances in parallel. For this to work, the orchestrator [storage configuration](../storage.md#redis-storage)
needs to be set to `redis` and there has to be a load balancer that forwards requests to the orchestrator instances.
