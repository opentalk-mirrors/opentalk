# Identity Provider

This page describes how the OIDC provider is configured in the controller.
Generic information for Keycloak and its configuration can be found in the [Keycloak section](keycloak.md).

## Configuration

In the past, configuration of OIDC and user search was done together within the [`keycloak`](keycloak_deprecated.md#deprecated-keycloak-configuration) section.
Starting with controller version 0.21.0, this is deprecated, support will be removed in the future.
It should be replaced with the separate [`oidc`](#configuration) and [`user_search`](user_search.md#user-search-configuration) sections.

The section in the [configuration file](configuration.md) is called `oidc`.

| Field        | Type                                                  | Required | Default value | Description                                                                                                                                              |
| ------------ | ----------------------------------------------------- | -------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `authority`  | `string`                                              | yes      | -             | Base url for the OIDC authority. Will be used for frontend and controller unless overwritten by `oidc.frontend.authority` or `oidc.controller.authority` |
| `frontend`   | [frontend configuration](#frontend-configuration)     | yes      | -             | Configuration dedicated to the frontend                                                                                                                  |
| `controller` | [controller configuration](#controller-configuration) | yes      | -             | Configuration dedicated to the controller                                                                                                                |

### Frontend configuration

| Field           | Type     | Required | Default value         | Description                                                                      |
| --------------- | -------- | -------- | --------------------- | -------------------------------------------------------------------------------- |
| `authority`     | `string` | no       | From `oidc.authority` | OIDC authority base url for the frontend                                         |
| `client_id`     | `string` | yes      | -                     | Client id that will be used by the frontend when connecting to the oidc provider |

### Controller configuration

| Field           | Type     | Required | Default value         | Description                                                                            |
| --------------- | -------- | -------- | --------------------- | -------------------------------------------------------------------------------------- |
| `authority`     | `string` | no       | From `oidc.authority` | OIDC authority base url for the controller                                             |
| `client_id`     | `string` | yes      | -                     | Client id that will be used by the controller when connecting to the oidc provider     |
| `client_secret` | `string` | yes      | -                     | Client secret that will be used by the controller when connecting to the oidc provider |

### Examples

#### Default Setup

```toml
[oidc]
authority = "https://localhost:8080/auth/realms/OPENTALK"

[oidc.frontend]
client_id = "Frontend"

[oidc.controller]
client_id = "Controller"
client_secret = "v3rys3cr3t"
```

## OIDC and User Info

The following fields returned by the OIDC provider's `userinfo` endpoint are used by the {{ product_name }} Controller. These fields differ for authentication of normal users and services.

> Note: Previously these fields were required to be in the ID Token's claims, but the ID Token is no longer required.

### User Info fields for user login

| Field           | Type       | Required                                                                      | Description                                                                                     |
| --------------- | ---------- | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `sub`           | `string`   | yes                                                                           | Unique identifier of the user                                                                   |
| `email`         | `string`   | yes                                                                           | E-Mail address of the user                                                                      |
| `given_name`    | `string`   | yes                                                                           | The given name (also known as first name) of the user                                           |
| `family_name`   | `string`   | yes                                                                           | The family name (also know as last name) of the user                                            |
| `tenant_id`     | `string`   | if [tenant `assignment`](../advanced/tenants.md) is `"by_external_tenant_id"` | Contains the identifier of the user's tenant                                                    |
| `tariff_id`     | `string`   | if [tariffs](../advanced/tariffs.md) are used                                 | The external id of the tariff. See [tariffs](../advanced/tariffs.md) for further details        |
| `tariff_status` | `string`   | if [tariffs](../advanced/tariffs.md) are used                                 | The external id of the tariff status. See [tariffs](../advanced/tariffs.md) for further details |
| `x_grp`         | `string[]` | no                                                                            | A list of groups which the user is part of                                                      |
| `phone_number`  | `string`   | no                                                                            | The phone number of the user                                                                    |
| `nickname`      | `string`   | no                                                                            | Nickname of the user, typically used to prefill the display name of a meeting participant       |
| `picture`       | `string`   | no                                                                            | URL to a user picture, will replace the gravatar url generation for that user if provided       |
| `zoneinfo`      | `string`   | no                                                                            | The timezone of the user, in IANA format (e.g. "Europe/Berlin")                                 |

#### Security considerations

For the `picture` field, the frontend will download the images found under the
provided URL. Therefore it is important to only provide URLs that are guaranteed
to not inject unwanted content, but rather have a policy which ensures that only
valid images are served.

### Access Token JWT fields for service login

| Field           | Type          | Required | Description                                       |
| --------------- | ------------- | -------- | ------------------------------------------------- |
| `exp`           | `string`      | yes      | RFC 3339 timestamp of the token's expiration date |
| `iat`           | `string`      | yes      | RFC 3339 timestamp of the token's issuing date    |
| `iss`           | `string`      | yes      | URL of the OIDC provider                          |
| `realm_access`  | `RealmAccess` | yes      | An object containing realm access information     |

The `RealmAccess` object contains these fields:

| Field   | Type       | Required | Description                                                       |
| ------- | ---------- | -------- | ----------------------------------------------------------------- |
| `roles` | `string[]` | yes      | A list of role identifiers that the service is allowed to provide |

The list of known service roles is:

- `"opentalk-call-in"`: The service is allowed to provide a meeting [phone call-in service](../advanced/call_in.md).
- `"opentalk-recorder"`: The service is allowed to provide a meeting [recording service](../advanced/additional_services/recorder.md).

## Back-channel logout

Starting with controller version 0.33.0 (OpenTalk 26.1.0), the controller implements
[OIDC Back-Channel Logout 1.0](https://openid.net/specs/openid-connect-backchannel-1_0.html).
When a user's session is terminated at the OIDC provider, the provider notifies the
controller so that the affected sessions are invalidated.

### Callback endpoint

The controller exposes the back-channel logout callback at:

```text
POST https://<controller-host>/v1/auth/logout
```

The request body uses the `application/x-www-form-urlencoded` content type and contains
a single `logout_token` parameter holding the logout token issued by the OIDC provider.
The controller validates the token (signature, expiration and the
`http://schemas.openid.net/event/backchannel-logout` event) and responds with
`204 No Content` on success, or `400 Bad Request` if the token is invalid.

The OIDC provider must be configured to call this URL on logout, i.e. the controller's
`…/v1/auth/logout` endpoint has to be registered as the client's back-channel logout URL.
See the [Keycloak section](keycloak.md#configuring-back-channel-logout) for a concrete
setup example.

### Token introspection requirement

The controller uses stateless authentication and does not maintain server-side sessions
keyed by a session ID. Therefore the controller resolves the affected session via the
`sub` (subject) claim of the logout token, and the `sid` (session ID) claim is
intentionally ignored.

To invalidate the affected sessions, the controller has to match the logged-out `sub`
against the access tokens it has cached, which means it must be able to determine the
`sub` of an access token. This requires the OIDC provider to **support [token
introspection](https://datatracker.ietf.org/doc/html/rfc7662)** or to issue access tokens
in JWT format — the same requirement that already applies to regular authentication (see
the [OIDC Authentication Flow](../under_the_hood/oidc_auth.md)). If the provider supports
neither introspection nor JWT access tokens, the `sub` cannot be resolved and
back-channel logout, like authentication itself, does not work.
