# OpenTalk Orchestrator

The Orchestrator manages scalable services in an OpenTalk deployment, similar to a loadbalancer. It proxies the internal
web APIs of orchestrated services and forwards requests towards the appropriate instance.

```ascii
                                                    ┌───────────┐
                                           ┌───────►│ Service A │
                      ┌────────────────┐   │        └───────────┘
┌────────────┐        │                ├───┘        ┌───────────┐
│ Controller ├───────►│  Orchestrator  ├───────────►│ Service A │
└────────────┘        │                ├───┐        └───────────┘
                      └────────────────┘   │        ┌───────────┐
                                           └───────►│ Service B │
                                                    └───────────┘
```

## Service endpoints

| Service       | Orchestrator endpoint |
| ------------- | --------------------- |
| Roomserver    | `/roomserver/`        |
| Recorder      | `/recording/`         |
| Transcription | `/transcription/`     |

## How it works

The orchestrator proxies the internal APIs of connected orchestrated services. For the API caller, the orchestrator
appears to be a single instance of the proxied service.

When a supported service is started and it has an orchestrator is configured, the service will attempt to register at
the orchestrator. Once registered, the service sends an initial dump of its metrics (current load, managed resources,
etc.) to the orchestrator.

The orchestrator maintains this list of registered services and continuously receives metrics from each of them. Based
on the managed resources of a service, the orchestrator can decide to which instance a request must be forwarded.

### Roomserver Example

Assume that a client wants to join room `28be42...`. The client sends its `/room/28be42.../start` request to the
controller. The controller checks the users permissions and requests a room token from its configured roomserver. In an
orchestrator deployment, the configured roomserver of the controller would be the `/roomserver` endpoint of the
orchestrator.

The orchestrator receives the request and can now determine which roomserver this request should be forwarded to. If
room `28be42...` is already managed by one of the roomservers, the request is forwarded to that roomserver instance.
If none of the roomservers manage room `28be42...`, the instance with the lowest load is selected to host the new room.

### Signaling Proxy

Besides the web API proxying, the orchestrator also provides a signaling endpoint under `/roomserver/v1/signaling/{token}`
that proxies the signaling websocket between a client and the managed roomserver instances.

Neither the client nor the roomserver need a any configuration for this. When a client requests a token from a
roomserver through the orchestrator, the orchestrator injects its signaling URL as `public_url` into the roomservers
`RoomServerAccess` response.

## Deployment Configuration

Each orchestrated service has an `orchestrator` section in their respective config that must be configured to point to
the orchestrator of that deployment.

The orchestrator must know the API keys of its orchestrated services. All API keys of deployed services must be added to
the `services.keys` field in the orchestrator configuration file. A service with an unknown API key id will have its
registration declined.

When configuring the controller, each service configuration should point to the respective orchestrator endpoint of that
service. For example, use orchestrators roomserver url (`<orchestrator_url>/roomserver`) as url for the controllers
roomserver configuration.

## Cluster setup

The orchestrator can scale horizontally by running multiple instances behind a load balancer. Some considerations
must be taken into account when running an orchestrators in a cluster.

### Load balancing

When running a cluster of orchestrators, an external load balancer must be configured to distribute requests to the
orchestrator instances. The load balancer must include WebSocket upgrade headers and proxy forwarding headers.

### Storage configuration

The orchestrator storage must be configured to use `Redis` as a backend:

```toml
[storage]
kind = "redis"
url = "redis://localhost:6379"
```

Redis needs to support `RESP3` protocol, so a minimum version of `7.0` is required.

The orchestrator does NOT support redis clusters due to limitations with scripts and notifications. This may change in
the future, but for now, only single redis instances are supported.

### Service API keys

All orchestrators in a cluster MUST have the same service API keys configured in order to allow services to register at
any orchestrator instance:

```toml
[services]
keys = [
    { id = "roomserver", secret = "secret" },           # ┐
    { id = "recorder", secret = "secret" },             # ├── These need to be the same for all orchestrator instances
    { id = "transcription", secret = "super_secret" },  # ┘
]
```

Missing or conflicting API keys will result in services not being able to consistently register at the orchestrator
cluster, leading to occasional errors in registration requests and services.
