# Access Control

{{ product_name }} enforces access to WebAPI endpoints in the controller's
authorization middleware. Each request is checked against the resource it
targets and the subject that authenticated for it (a logged-in user, a guest, or both). The rules that describe who may access
which endpoint are hard-coded per resource and evaluated on demand from the
controller database — there is no separate ACL that needs to be built up,
kept in memory, or synchronised between controllers. See
[Handling of WebAPI HTTP Requests](../under_the_hood/http_requests.md) for how
the middleware fits into the request pipeline.

## Configuration

Access control is not configurable. Earlier controller releases exposed an
`[authz]` (later `[authorization]`) section with a `synchronize_controllers`
option; this section is no longer read and can be removed from existing
`controller.toml` files. Leaving it in place will produce a warning about an
unknown configuration section on startup.

## Removed subcommands

The following `opentalk-controller` subcommands used to manage the in-memory
ACL and have been removed:

- `opentalk-controller acl`
- `opentalk-controller fix-acl`

Neither is necessary any more, because the authorization middleware derives
its decisions directly from the database tables that already model the authoritative
state (users, rooms, events, invites and tariffs).
