# Identity Provider (Keycloak)

The {{ product_name }} Controller uses [Keycloak](https://www.keycloak.org/), an OpenID Connect compatible
identity and access management software for single sign-on.

## Configuring Keycloak for {{ product_name }} Controller

!!! note

    The Keycloak user interface changed in the past and because of that it's safe to assume
    that it will continue to change moving forward. Instead of screenshots we describe what needs to be
    done, and link to the Keycloak documentation where needed. These links
    reference a specific version of Keycloak. If those settings are outdated, please refer to the
    [Keycloak documentation archive](https://www.keycloak.org/documentation-archive.html)
    and find the corresponding section there.

This manual describes the configuration for the {{ product_name }} Controller only, other
{{ product_name }} components might need separate configuration.

1. Create a [realm](https://www.keycloak.org/docs/latest/server_admin/index.html#proc-creating-a-realm_server_administration_guide) for usage with {{ product_name }} if it hasn't been created yet.
   - The **Realm ID** will be used in the `keycloak.realm` configuration field.
2. Create an [OpenID Connect client](https://www.keycloak.org/docs/latest/server_admin/index.html#proc-creating-oidc-client_server_administration_guide).
   - The **Client ID** will be used in the `keycloak.client_id` configuration field.
   - Enable **Client authentication** and **Service account roles** in the [Capability Config](https://www.keycloak.org/docs/latest/server_admin/index.html#capability-config).
3. Create [Confidential client credentials](https://www.keycloak.org/docs/latest/server_admin/index.html#_client-credentials).
   - Use the Client Authenticator **Client Id and Secret** .
   - The **Client secret** will be used in the `keycloak.client_secret` configuration field.

## Controller configuration

In the past, the OIDC and user search section in the [configuration file](./configuration.md) was called [`keycloak`](./keycloak_deprecated.md#deprecated-keycloak-configuration).
Starting with controller version 0.21.0, this is deprecated, support will be removed in the future.
Use the separate [`oidc`](./oidc.md#configuration) and [`user_search`](./user_search.md#user-search-configuration) sections instead.

## Token introspection

To authenticate WebAPI requests, the {{ product_name }} Controller must verify the access
tokens it receives. It uses [token introspection](https://datatracker.ietf.org/doc/html/rfc7662)
whenever the OIDC provider offers an introspection endpoint, and only falls back to verifying
access tokens locally as JWTs when introspection is unavailable.

Introspection is preferred because it additionally reveals whether a token is still active, for
example whether it has been revoked. Local JWT verification can only check the token's signature
and expiration and therefore cannot detect revoked tokens.

Keycloak exposes the introspection endpoint for confidential clients out of the box. A client
configured with **Client authentication** enabled (as described above) therefore supports
introspection without further configuration.

!!! warning

    If the identity provider supports neither token introspection nor JWT access tokens, the
    controller cannot verify access tokens and rejects every request. See the
    [OIDC Authentication Flow](../under_the_hood/oidc_auth.md) for details.

### Audience requirement (Keycloak 26.6.2 and newer)

Since Keycloak **26.6.2**, token introspection only succeeds when the controller's confidential
client is listed in the token's `aud` claim. Tokens issued for other clients no longer contain it
by default, so the controller rejects them with an error that looks like an expired token.

This only affects the **frontend client** (e.g. `Frontend`), because the controller introspects the
user access tokens issued for it. Add an **Audience** mapper to that client (client scope →
**Mappers** → **Add mapper** → **By configuration** → **Audience**) with **Included Client
Audience** set to the controller's client.

## Configuring back-channel logout

The {{ product_name }} Controller implements [OIDC back-channel logout](./oidc.md#back-channel-logout).
To let Keycloak notify the controller when a user logs out, configure the controller's
client accordingly.

1. Open the [OpenID Connect client](https://www.keycloak.org/docs/latest/server_admin/index.html#proc-creating-oidc-client_server_administration_guide) you created for the controller.
2. In the client's **Logout settings**, set the **Backchannel logout URL** to the controller's callback endpoint `https://<controller-host>/v1/auth/logout`. Replace `<controller-host>` with the public host (and path prefix, if any) under which the controller's API is reachable.
3. Leave **Backchannel logout session required** at whatever value you prefer. The controller resolves sessions via the `sub` claim and ignores `sid`, so this setting does not affect logout on the controller side.
4. Make sure [token introspection](#token-introspection) is available for the client. The controller resolves the affected session from the access token's `sub` claim, so it relies on the same token verification as regular authentication.
