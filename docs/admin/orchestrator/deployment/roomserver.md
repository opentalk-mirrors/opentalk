# Roomserver scaling

Since version `26.2`, the roomserver is mandatory in an OpenTalk deployment. The roomserver handles the meeting room
state and client signaling. When configured, the orchestrator proxies its Web API as well as the signaling connection.

## Configuration

1. Add the `orchestrator` section to the `roomserver.toml`:

   ```toml
   [orchestrator]
   # The API key of the orchestrator, one of the orchestrator's `http.api_keys`
   api_key = { id = "orchestrator", secret = "secret" }
   # The URL of the orchestrator
   url = "<orchestrator-url>"
   ```

2. Update the Controller's RoomServer **and [LiveKit](#livekit-signaling)** configuration as shown in the [Controller's roomserver](controller.md#roomserver) section

3. Add the roomservers API key to the `services.keys` section in the `orchestrator.toml` (See [Services](../services.md))

The roomserver will then register at the configured orchestrator on startup. Once registered, it is part of the
roomserver cluster.

## Signaling insights

When a client requests a roomserver token, the request gets routed through the orchestrator. The token response from the
roomserver includes its signaling URL. The orchestrator replaces the signaling url in that response with its own public
URL.

This means that the orchestrator will be the entrypoint for signaling connections in a deployment where roomservers
are scaled.

!!! important

    Make sure that the public URL of the orchestrator is reachable by clients

### LiveKit signaling

LiveKit is a special case. It is not scaled through the orchestrator, but its signaling must still be proxied by the
orchestrator when scaling the RoomServer.

Each RoomServer proxies the LiveKit signaling endpoint (`/rtc`) for its rooms. Running multiple RoomServers would result
in multiple LiveKit signaling endpoints that need to be reachable by clients, one for each RoomServer. To avoid this,
the orchestrator provides a LiveKit proxy under `/livekit/` that forwards LiveKit signaling requests to the correct
RoomServer instance.

Set the LiveKit public URL in the `controller.toml` to the orchestrator's [public URL](../http.md#public-url):

`controller.toml`:

```toml
[roomserver.modules.livekit]
# Clients connect to LiveKit through the orchestrator
public_url = "<orchestrator-public-url>/livekit/"
# ...
```
