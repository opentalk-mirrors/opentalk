# User Search

{{ product_name }} can search for users and display suggestions when attempting to
invite users into meetings. User search can be configured to behave differently
depending on the use case.

The HTTP endpoint that is used by frontend to query users is `GET /users/find`.

When inviting users by their E-Mail address, the user search backend is
also used to determine whether a user is known. This happens in the `POST
/events/{event_id}/invites` endpoint.

## User search configuration

| Field                 | Type     | Required | Default value                                                                      | Description                                                                                                                |
| --------------------- | -------- | -------- | ---------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `users_find_behavior` | `enum`   | no       | `"from_user_search_backend"` if `backend` field is present, `"disabled"` otherwise | Sets the behaviour of the `/users/find` endpoint. See [Users find endpoint behavior](#users-find-endpoint-behavior) below. |
| `backend`             | `enum`   | no       | -                                                                                  | Defines which backend to use for user search. See [User search backends](#user-search-backends) below.                     |

The whole `[user_search]` section is optional. If it is absent, then user search
is disabled entriely (equivalent to `users_find_behavior = "disabled"` and
`backend` field absent.

### Users find endpoint behavior

The `/users/find` endpoint can be configured with `users_find_behavior`.

Available values:

- `"disabled"`: Don't use user search at all in this endpoint. The `/users/find`
  endpoint will return a `404 NOT FOUND` HTTP status code.
  This is equivalent to what used to be `disable_users_find = true` in the
  [`[endpoints]` section](./endpoints.md).
- `"from_database"`: Use the {{ product_name }} database to search for user accounts,
  but don't request any information from the user search backend, even if it is
  configured in the `backend` field.
- `"from_user_search_backend"`: Use the {{ product_name }} database to search for user
  accounts first, and request information from the user search backend configured
  in the `backend` field in addition. Search results may include users that were
  never registered on the {{ product_name }} Controller.

#### Default value

If the `backend` field is present, each endpoint for which no
`*_behavior` field has been configured will behave as if the value was
`"from_user_search_backend"`.

If no `backend` field is present, each endpoint for which no `*_behavior` field
has been configured will behave as if the value was `"disabled"`.

#### Event invite endpoint

{{ product_name }} can be configured to allow inviting guests through external email
addresses, even if they can not register, e.g. because the instance is limited
to accounts of an organization. These guests will then receive an email with an
invite link they can use to join a meeting.

This behavior can be enabled by setting
`endpoints.event_invite_external_email_address` to `true` in the
[HTTP endpoints configuration](./endpoints.md).

Because not all potential users might have logged in to {{ product_name }}, these cannot
be found in its database. So if invitation of external E-Mail addresses
is disabled, a method is necessary to determine whether these users can be
considered as known, and therefore invited by E-Mail. The user search backend
configured in the `backend` field will be queried for that.

## User search backends

The `backend` field is optional. If a backend is selected by setting one of
the values described below, then additional fields may be required or allowed
depending on the backend.

### Not using any search backend

If the `backend` field is absent, no search backend will be queried for user information.

In this case, the `users_find_behavior` field must:

- either be absent
- or be set to `"disabled"` (equivalent to being absent)
- or be set to `"from_database`"

Setting `users_find_behavior` to `"from_user_search_backend"` is an error if the
`backend` field is absent.

#### Example

Searching for users in the database:

```toml
[user_search]
users_find_behavior = "from_database"
```

Disabling search entirely (equivalent to not having a `[user_search]` section at all:

```toml
[user_search]
users_find_behavior = "disabled"
```

### Keycloak user search backend

The {{ product_name }} Controller can search on a Keycloak instance if configured
correctly. This does not happen through OIDC, but instead uses the Keycloak web
api to call endpoints there.

When setting `backend = "keycloak_webapi"`, these additional fields can be configured
in the `[user_search]` section:

| Field                             | Type     | Required | Default value                        | Description                                                                                                          |
| --------------------------------- | -------- | -------- | ------------------------------------ | -------------------------------------------------------------------------------------------------------------------- |
| `api_base_url`                    | `string` | yes      | -                                    | Base URL of the Keycloak web api                                                                                     |
| `client_id`                       | `string` | no       | From `oidc.controller.client_id`     | Client id that is used to authenticate against the user search API                                                   |
| `client_secret`                   | `string` | no       | From `oidc.controller.client_secret` | Client secret that is used to authenticate against the user search API                                               |
| `external_id_user_attribute_name` | `string` | no       | See below                            | The attribute by which Keycloak and {{ product_name }} users are assigned to each other. See below for more details. |

The `external_id_user_attribute_name` setting is used to configure how Keycloak users resulting from a search and registered
Opentalk users are assigned to each other.

The following assignment strategies are available:

- by Keycloak id (default): This is used if `external_id_user_attribute_name` is not set. Keycloak users are assigned to
  Opentalk users using Keycloak's id field.
- by user attribute: Keycloak must provide a user attribute holding the user IDs. The name of this user attribute must be
  set here in `external_id_user_attribute_name`.

#### Configuring Keycloak for user search

!!! note

    The Keycloak user interface changed in the past and because of that it's safe to assume
    that it will continue to change moving forward. Instead of screenshots we describe what needs to be
    done, and link to the Keycloak documentation where needed. These links
    reference a specific version of Keycloak. If those settings are outdated, please refer to the
    [Keycloak documentation archive](https://www.keycloak.org/documentation-archive.html)
    and find the corresponding section there.

1. Configure a client which will be used to access the Keycloak web api. Details
    about that can be found in the [Keycloak section](keycloak.md).
2. For the [OpenID Connect client](https://www.keycloak.org/docs/latest/server_admin/index.html#proc-creating-oidc-client_server_administration_guide)
    enable **Service account roles** in the [Capability Config](https://www.keycloak.org/docs/latest/server_admin/index.html#capability-config).
3. In the [Service account](https://www.keycloak.org/docs/latest/server_admin/index.html#_service_accounts) add these service account roles:
    - `realm-management` *query-users*
    - `realm-management` *view-users*

#### Example

Configure searching through the Keycloak web api, using the same OIDC client as configured in `[oidc.controller]`:

```toml
[user_search]
backend = "keycloak_webapi"
api_base_url = "https://localhost:8080/auth/admin/realms/OPENTALK"
users_find_behavior = "from_user_search_backend"
```

Configure searching through the Keycloak web api, with a different OIDC client than what is configured in `[oidc.controller]`:

```toml
[user_search]
backend = "keycloak_webapi"
api_base_url = "https://localhost:8080/auth/admin/realms/OPENTALK"
client_id = "ControllerUserSearch"
client_secret = "v3rys3cr3t"
users_find_behavior = "from_user_search_backend"
```

Configure searching only in the database, but use the Keycloak web api for
resolving allowed users in the `POST /events/{event_id}/invites` endpoint:

```toml
[user_search]
backend = "keycloak_webapi"
api_base_url = "https://localhost:8080/auth/admin/realms/OPENTALK"
users_find_behavior = "from_database"
```

Disable search suggestions, but use the Keycloak web api for resolving allowed
users in the `POST /events/{event_id}/invites` endpoint:

```toml
[user_search]
backend = "keycloak_webapi"
api_base_url = "https://localhost:8080/auth/admin/realms/OPENTALK"
users_find_behavior = "disabled"
```
