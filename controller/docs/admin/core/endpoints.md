# HTTP Endpoints

The behavior of some API endpoints of the {{ product_name }} Controller can be modified.

## Configuration

The section in the [configuration file](./configuration.md) is called `endpoints`.

| Field                                 | Type   | Required | Default value | Description                                                                                                                                                                          |
| ------------------------------------- | ------ | -------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `event_invite_external_email_address` | `bool` | no       | false         | Affects the `POST /events/{event_id}/invites` endpoint and allows users to invite email addresses that are unknown to the Controller or the [user search backend](./user_search.md). |
| `disallow_custom_display_name`        | `bool` | no       | false         | Enforces the display name that was provided by Keycloak and disallows users to change their display names via the `PATCH /users/me` endpoint.                                        |
| `disable_openapi`                     | `bool` | no       | false         | Disables the `GET /v1/openapi.json` and `GET /swagger` endpoints which serve information about the {{ product_name }} controller WebAPI.                                             |
| `disable_users_find`                  | `bool` | no       | false         | :warning: Deprecated. Configure [user search](./user_search.md) instead.                                                                                                             |
| `users_find_use_kc`                   | `bool` | no       | false         | :warning: Deprecated. Configure [user search](./user_search.md) instead.                                                                                                             |

For configuring user search, see the [User search section](./user_search.md).

### Examples

#### Default setup

```toml
[endpoints]
event_invite_external_email_address = false
disallow_custom_display_name = false
disable_openapi = false
```
