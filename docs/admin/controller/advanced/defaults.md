# Default and Fallback Values

## Features

In the [configuration file](../core/configuration.md), the format of a [`feature`](./modules.md#features) is: `[<module>::]<feature>`.
A missing module specifier defaults to `"core"`. The features currently supported are:

- `core::call_in`
- `integration::outlook`

The [`modules`](./modules.md#opentalk-controller-modules-subcommand) subcommand outputs all modules
available in the {{ product_name }} controller, including the features that can be enabled or disabled.

## Configuration

The section in the [configuration file](../core/configuration.md) is called `defaults`.

| Field                              | Type                     | Required | Default value | Description                                                                  |
| ---------------------------------- | ------------------------ | -------- | ------------- | ---------------------------------------------------------------------------- |
| `user_language`                    | `string`                 | no       | `"en-US"`     | Default language of a new user                                               |
| `screen_share_requires_permission` | `bool`                   | no       | `false`       | When `true`, screen sharing requires explicit permission                     |
| `timezone`                         | `string`                 | no       | see below     | The global timezone of the controller, in IANA format (e.g. "Europe/Berlin") |
| `disabled_features`                | `string[]`               | no       | `[]`          | A list of disabled features in the controller                                |
| `room_alias`                       | [RoomAlias](#room-alias) | no       | see below     | Room alias related settings.                                                 |

The `timezone` field sets the timezone used by the system and as the users' default. If not set here, the `TZ` environment variable and the operating system
are consulted in this order, finally falling back to "Etc/UTC").

### Room Alias

Users can create rooms with a personal name, which can be used instead of the room ID in the room URL.
The `room_alias` field allows to configure the default behavior of room aliases.
By default, a room alias has a suffix appended to the room name, which is a randomly generated string of 16 characters.
This serves two purposes:

1. It prevents the room URL from being guessed by malicious users.
2. It ensures the alias is always unique without restricting the available room aliases names.

The generation of suffixes can be disabled by setting `disable_suffix` to `false`.
This will allow any uninvited or unregistered person to join a room with an alias, unless the room is secured with a password.
This is considered **insecure** when the OpenTalk instance is publicly reachable and should only be used in trusted environments.
This will further restrict the available room alias names to unique names, meaning that a room alias can only be used once and can only be reused after the room has been deleted.

The length of the suffix can be changed by setting `suffix_length` to a value between 8 and 64.
Short suffixes are vulnerable to brute-force attacks, so it is recommended to use a length of at least 16 characters.

| Field            | Type   | Required | Default value | Description                                                     |
| ---------------- | ------ | -------- | ------------- | --------------------------------------------------------------- |
| `disable_suffix` | `bool` | no       | `false`       | Whether or not suffixes are appended to the personal room name. |
| `suffix_length`  | `u8`   | no       | `16`          | The number of characters a suffix has.                          |

### Examples

#### Set the global timezone and disable the `core::call_in` and `integration::outlook` features

```toml
[defaults]
timezone = "Europe/Berlin"
disabled_features = ["core::call_in", "integration::outlook"]
```

#### Set default user language, require explicit screen share permissions and disable the `core::call_in` feature

```toml
[defaults]
user_language = "de-DE"
screen_share_requires_permission = true
disabled_features = ["call_in"]
```
