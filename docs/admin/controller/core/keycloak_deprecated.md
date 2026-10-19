# Deprecated: Identity Provider (Keycloak)

This page describes how OIDC using Keycloak was configured in the controller.
Generic information for Keycloak and its configuration can be found in the [Keycloak section](./keycloak.md).

## Deprecated Keycloak configuration

!!! warning

    In the past, the OIDC and user search section in the [configuration file](./configuration.md) was called [`keycloak`](#deprecated-keycloak-configuration).
    Support will be removed in the future, use the separate [`oidc`](./oidc.md#configuration) and [`user_search`](./user_search.md#user-search-configuration)
    sections instead.

The section in the [configuration file](configuration.md) was called `keycloak`.

| Field                               | Type     | Required | Default value | Description                                                                                                                            |
| ----------------------------------- | -------- | -------- | ------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `base_url`                          | `string` | yes      | -             | The URL where the Keycloak server can be reached                                                                                       |
| `realm`                             | `string` | yes      | -             | The name of the default Keycloak realm, read more on [Keycloak](https://www.keycloak.org/docs/latest/server_admin/#configuring-realms) |
| `client_id`                         | `string` | yes      | -             | The unique identifier for the {{ product_name }} client                                                                                |
| `client_secret`                     | `string` | yes      | -             | The secret corresponding to the specified client ID                                                                                    |
| `external_id_user_attribute_name`   | `string` | no       | See below     | The attribute by which Keycloak and {{ product_name }} users are assigned to each other. See below for more details.                   |

For configuring user search, see the [User search section](./user_search.md).

The `external_id_user_attribute_name` setting is used to configure how Keycloak users resulting from a search and registered
Opentalk users are assigned to each other.
The following assignment strategies are available:

- by Keycloak id (default): This is used if `external_id_user_attribute_name` is not set. Keycloak users are assigned to
  Opentalk users using Keycloak's id field.
- by user attribute: Keycloak must provide a user attribute holding the user IDs. The name of this user attribute must be
  set here in `external_id_user_attribute_name`.

### Examples

#### Deprecated default setup

```toml
[keycloak]
base_url = "http://localhost:8080/auth"
realm = "MyRealm"
client_id = "Controller"
client_secret = "c64c5854-3f02-4728-a617-bbe98ec42b8f"
```
