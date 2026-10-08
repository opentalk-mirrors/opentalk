# Controller configuration

The controller does not have a dedicated orchestrator settings section. Instead, the URLs of proxied services must be
replaced with the URL of the orchestrator.

The controller is generally unaware of the orchestrator, from its perspective, the orchestrator acts as if it was the
proxied service.

## Roomserver

The orchestrator proxies the RoomServer's WebAPI as well as the signaling connection.

To proxy the RoomServer, change the `service_url` and `api_key` in the controller's `roomserver` section to
the Orchestrator's URL and api key:

`controller.toml`:

```toml
[roomserver]
# The orchestrator cannot proxy an internal RoomServer
kind = "external"

# The orchestrator's URL that proxies the RoomServer
service_url = "http://localhost:11222/roomserver/"

# The orchestrator's API key
api_key = { id = "orchestrator", secret = "secret" }
# ...
```

!!! warning

    When proxying the RoomServer, the Orchestrator must also proxy LiveKit signaling (See [LiveKit](#livekit))

The recorder and transcription service are configured the same way in their module sections:

## Recorder

Replace the `url` and `api_key` with the orchestrators api key and recorder proxy endpoint.

`controller.toml`:

```toml
[roomserver.modules.recording]
url = "http://localhost:11222/recording/"
api_key = { id = "orchestrator", secret = "secret" }
```

## Transcription

Replace the `url` and `api_key` with the orchestrators api key and transcription proxy endpoint.

`controller.toml`:

```toml
[roomserver.modules.transcription]
url = "http://localhost:11222/transcription/"
api_key = { id = "orchestrator", secret = "secret" }
```

## LiveKit

!!! warning

    When proxying the RoomServer, the LiveKit signaling **must** be proxied by the orchestrator as well. The endpoint
    `<public_url>livekit/` must be reachable by clients.

Set the LiveKit public URL in the controller to the orchestrator's [public URL](../http.md#public-url), include the
`livekit/` path segment:

`controller.toml`:

```toml
[roomserver.modules.livekit]
# Clients connect to LiveKit through the orchestrator
public_url = "<orchestrator-public-url>/livekit/"
# ...
```
