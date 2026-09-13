# Tariffs

Tariffs in {{ product_name }} can be created, edited, and deleted with the commandline using `opentalk-controller tariffs <COMMAND>`.

A tariff can restrict the {{ product_name }} resource usage by imposing a combination of different quota types.
The following quotas are supported:

| Quota Name               | Description                                                                                                                             |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------- |
| `max_storage`            | The maximum allowed storage per user in bytes. This is a soft limit. When the limit is exceeded, a user can not store additional files. |
| `room_time_limit_secs`   | This quota restricts the total duration for which a tenant can utilize a meeting room, measured in seconds.                             |
| `room_participant_limit` | This quota sets a limit on the number of participants that can join a room.                                                             |

## Configuration

The section in the [configuration file](../core/configuration.md) is called `tariffs`.

| Field                | Type     | Required | Default value             | Description                                                                          |
| -------------------- | -------- | -------- | ------------------------- | ------------------------------------------------------------------------------------ |
| `assignment`         | `string` | no       | `"static"`                | The tariff assignment strategy. Either `"static"` or `"by_external_tariff_id"`       |
| `static_tariff_name` | `string` | no       | `"OpenTalkDefaultTariff"` | Name of the tariff assigned to every user. Only used when `assignment` is `"static"` |

When `assignment` is set to `"by_external_tariff_id"`, the OIDC provider (Keycloak) must be configured to include a `tariff_id` field in its ID token's JWT claims. It is used to assign users the correct tariff.

### `tariffs.status_mapping`

Status mapping can only be used when the tariff assignment is configured as `"by_external_tariff_id"`. If present, the controller will look at the JWT attribute named `tariff_status` and transfer its value to its internal tariff status based on the values of the `default`, `paid` and `downgraded` field values. An entry in any of the lists below must be unique across all lists.

| Field                    | Type       | Required | Default value | Description                                                                                      |
| ------------------------ | ---------- | -------- | ------------- | ------------------------------------------------------------------------------------------------ |
| `downgraded_tariff_name` | `string`   | no       | -             | The name of the tariff that gets applied when the user's tariff status is `"downgraded"`.        |
| `default`                | `string[]` | no       | -             | List of status values that map to the default tariff status.                                     |
| `paid`                   | `string[]` | no       | -             | List of status values that indicate the user's tariff has been paid and is valid.                |
| `downgraded`             | `string[]` | no       | -             | List of status values that indicate the user's tariff is downgraded (e.g. because it is unpaid). |

Any user with an invalid value in the `tariff_status` attribute will be set to the default status, but a warning will be issued if the mapping does not contain that attribute value.

## Assigning Tariffs to Users via Keycloak

Tariffs can be assigned to individual users in Keycloak. To do so, navigate to the user's **Attributes** section in the Keycloak admin console and add an attribute with the key `tariff_id` and a value matching the external tariff ID of the desired tariff.

For this to take effect, the controller must be configured to use the `"by_external_tariff_id"` assignment strategy.

!!! danger

    When externally assigning a tariff to a user that affects authorization (e.g. `core::guests_allowed`), the changes only take effect after the user has logged in again.
    This includes changes to any events created by the user.

## `opentalk-controller tariffs` subcommand

This subcommand is used to manage tariffs.

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-tariffs-help.md -->

```text
Manage tariffs

Usage: opentalk-controller tariffs <COMMAND>

Commands:
  list    List all available tariffs
  create  Create a new tariff
  delete  Delete a tariff by name
  edit    Modify an existing tariff
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-tariffs-help.md -->

### Examples

#### List All Tariffs

Run `opentalk-controller tariffs list` to show all existing tariffs.

#### Create a New Tariff

Run `opentalk-controller tariffs create <TariffName> <ExternalTariffId>` to create a new tariff.

<!-- begin:fromfile:cli-usage/opentalk-controller-tariffs-create.md -->

```text
Create a new tariff

Usage: opentalk-controller tariffs create [OPTIONS] <TARIFF_NAME> <EXTERNAL_TARIFF_ID>

Arguments:
  <TARIFF_NAME>         Name of the tariff
  <EXTERNAL_TARIFF_ID>  Eternal ID to map to the tariff

Options:
      --disabled-modules <DISABLED_MODULES>    Comma-separated list of modules to disable
      --disabled-features <DISABLED_FEATURES>  Comma-separated list of features to disable
      --quotas <QUOTAS>                        Comma-separated list of key=value pairs
  -h, --help                                   Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-tariffs-create.md -->

#### Delete an Existing Tariff

Run `opentalk-controller tariffs delete <TariffName> <ExternalTariffId>` to delete an existing tariff.

<!-- begin:fromfile:cli-usage/opentalk-controller-tariffs-delete.md -->

```text
Delete a tariff by name

Usage: opentalk-controller tariffs delete <TARIFF_NAME>

Arguments:
  <TARIFF_NAME>  Name of the tariff to delete

Options:
  -h, --help  Print help
```

<!-- end:fromfile:cli-usage/opentalk-controller-tariffs-delete.md -->

#### Edit an Existing Tariff

Run `opentalk-controller tariffs edit <TariffName>` to edit an existing tariff.

!!! danger

    When changing disabled features that affect authorization (e.g. `core::guests_allowed`), the controller has to be restarted for the changes to take effect in the authorization.

Help output looks like this:

<!-- begin:fromfile:cli-usage/opentalk-controller-tariffs-edit.md -->

```text
Modify an existing tariff

Usage: opentalk-controller tariffs edit [OPTIONS] <TARIFF_NAME>

Arguments:
  <TARIFF_NAME>
          Name of the tariff to modify

Options:
      --set-name <SET_NAME>
          Set a new name

      --add-external-tariff-ids <ADD_EXTERNAL_TARIFF_IDS>
          Comma-separated list of external tariff_ids to add

      --remove-external-tariff-ids <REMOVE_EXTERNAL_TARIFF_IDS>
          Comma-separated list of external tariff_ids to remove

      --add-disabled-modules <ADD_DISABLED_MODULES>
          Comma-separated list of module names to add

      --remove-disabled-modules <REMOVE_DISABLED_MODULES>
          Comma-separated list of module names to remove

      --add-disabled-features <ADD_DISABLED_FEATURES>
          Comma-separated list of feature names to add

      --remove-disabled-features <REMOVE_DISABLED_FEATURES>
          Comma-separated list of feature names to remove

      --add-quotas <ADD_QUOTAS>
          Comma-separated list of key=value pairs to add, overwrites quotas with the same name

      --remove-quotas <REMOVE_QUOTAS>
          Comma-separated list of quota keys to remove

          Possible values:
          - max_storage:            This quota limits the total amount of data, measured bytes, that can be stored by the tenant. This is a soft limit which allows the user to store files as long as their usage is below the limit. Once the limit is reached or exceeded, no new data can be stored
          - room_time_limit_secs:   This quota restricts the total duration for which a tenant can utilize a meeting room, measured in seconds
          - room_participant_limit: This quota sets a limit on the number of participants that can join a room

  -h, --help
          Print help (see a summary with '-h')
```

<!-- end:fromfile:cli-usage/opentalk-controller-tariffs-edit.md -->

These subcommand options enable the modification of tariff names, external tariff IDs, disabled modules and features as well as quotas.
