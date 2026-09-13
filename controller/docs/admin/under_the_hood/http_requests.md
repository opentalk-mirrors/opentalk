# Handling of WebAPI HTTP Requests

Requests to the WebAPI are handled by the web server which is embedded in the
{{ product_name }} controller. Each request is passed through some layers of middleware
before they reach the handler that processes and answers it.

```mermaid
flowchart TD;
  Client -- Request --> Server[HTTP Server];
  Server --> Headers>Header request id middleware];
  Headers --> Tracing>Tracing middleware];
  Tracing --> Cors>CORS enforcement middleware];
  Cors --> RequestMetrics>Request metrics middleware];
  RequestMetrics --> IsSignaling{Signaling?};
  IsSignaling -- yes --> Signaling[Signaling endpoint handler];
  IsSignaling -- no --> IsMetrics{Metrics?};
  IsMetrics -- yes --> Metrics[Metrics endpoint handler];
  IsMetrics -- no --> IsPublic{Public?};
  IsPublic -- yes --> PublicApiEndpoints[[Public API endpoint handlers]];
  IsPublic -- no --> IsService{Service?};
  IsService -- yes --> ServiceAuth>Service authentication middleware];
  ServiceAuth --> ServiceEndpoints[[Service endpoint handlers]];
  IsService -- no --> OIDC>OIDC middleware];
  OIDC --> Authorization>Authorization middleware];
  Authorization--> AuthenticatedApiEndpoints[[Authenticated API endpoint handlers]];
```

## Details

### HTTP server

The HTTP server receives the requests from the client and passes it on to the
middlewares and endpoints.

### Header request id middleware

This middleware adds a unique id to the request so that it can be tracked
throughout the system. If the same header was already sent from the client, it
will not be modified.

### Tracing middleware

The [tracing logger](https://docs.rs/tracing-actix-web/latest/tracing_actix_web/struct.TracingLogger.html)
middleware captures structured request and response information and passes it on
to the configured tracing subscriber. It does not apply any modifications to
the request.

### CORS enforcement middleware

This middleware refuses or allows requests based on the CORS policy of the web
service. It does not apply any modifications to the request.

### Request metrics middleware

This middleware records the handler duration for each request handler
categorized by the HTTP response status. It does not apply any modifications to
the request.

### Signaling endpoint handler

Handles the requests to the WebSocket signaling endpoint which is responsible
for processing the communication between clients and the controller during
a meeting. The documentation of the protocol can be found in the
[Signaling section of the developer documentation](https://docs.opentalk.eu/developer/controller/signaling/).

### Metrics endpoint handler

Endpoint to fetch collected metrics from the controller. The documentation can
be found in the [metrics section](../core/logging/metrics.md#web-api).

Depending on the corresponding configuration the endpoint handler itself will
either return the contents or deny the request with the corresponding HTTP
status code.

### Public API endpoint handlers

These are API endpoints that are available to the public without authentication.
They provide general information that the client needs to fetch upfront before
logging in.

The endpoints are described in the
[OpenAPI specification](https://docs.opentalk.eu/developer/controller/rest/).

### Service auth middleware

Authenticates [services](#service-endpoint-handlers) using the claims provided
in the OIDC access token.

### Service endpoint handlers

These endpoints are for services that log in to meetings for providing
functionality for the meeting, such as the
[{{ product_name }} Recorder](https://docs.opentalk.eu/admin/recorder/) or the
[{{ product_name }} Obelisk](https://docs.opentalk.eu/admin/obelisk/).

### OIDC middleware

This middleware verifies that the OIDC JSON Web Token provided by the client is
valid and contains the required information. It extracts information such as
the user id and adds it to the request for the subsequent middlewares or request
handlers.

In addition, it can authenticate through an invite code header for guest
participants when no OIDC token is provided.

### Authorization middleware

The authorization middleware enforces access rules to the authenticated
endpoints. For each request it classifies the target endpoint as a resource
(e.g. a specific event or room) and asks the controller whether any of the
authenticated subjects — the logged-in user, a guest presenting an invite
code, or both — is allowed to access it with the requested HTTP method.
Decisions are computed on demand from the controller database, so no ACL
cache is kept and no synchronisation between controllers is required. When an
endpoint is not allowed for access, the corresponding HTTP error status code
is returned.

### Authenticated API endpoint handlers

These are API endpoints that are only available after authentication.

The endpoints are described in the
[OpenAPI specification](https://docs.opentalk.eu/developer/controller/rest/).
