# Configuration

When the recorder gets started, it loads the configuration from the
environment. It reads the settings in this order:

- Read environment variables which have a specific name, see section
  [Environment variables](#environment-variables).
- Load from a configuration file. Unless a path is given explicitly via `--config`/`-c`, the first of the following locations that exists is used:
    - `recorder.toml` in the current working directory
    - `<XDG_CONFIG_HOME>/opentalk/recorder.toml` (usually `~/.config/opentalk/recorder.toml`)
    - `/etc/opentalk/recorder.toml`

## Sections in the configuration file

Functionality that can be configured through the configuration file:

- [Controller](controller.md)
- [HTTP](http.md)
- [Monitoring](monitoring.md)
- [Orchestrator](orchestrator.md)
- [Recorder](recorder.md)

## Environment variables

Settings in the configuration file can be overwritten by environment variables,
nested fields are separated by two underscores `__`. The pattern looks like
this:

```sh
OPENTALK_REC_<field>__<nested-field>…
```

### Limitations

Some settings can not be overwritten by environment variables. This is for
example the case for entries in lists, because there is no environment variable
naming pattern that could identify the index of the entry inside the list.

### Examples

In order to set the `controller.url` field, this environment variable could be used:

```sh
OPENTALK_REC_CONTROLLER__URL=http://localhost:11311
```

## Example configuration file

This file can be found in the source code distribution under `extra/example.toml`

<!-- begin:fromfile:toml:config/example.toml -->

```toml
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

[monitoring]
port = 11411

[controller]
# The URL of the controller
url = "http://localhost:11311"
# The API to access the controller
api_key = { "id" = "controller", "secret" = "secret" }

# Optional orchestrator configuration
#[orchestrator]
# The API key of the orchestrator
#api_key = { id = "orchestrator", secret = "secret" }
# The orchestrator URL
#url = "http://127.0.0.1:11222"

[http]
# The address to bind the HTTP Server to (defaults to 0.0.0.0).
addr = "0.0.0.0"
# The port to bind the HTTP Server to (defaults to 11511).
port = 11511

# The api keys for internal service endpoints
#
# The recorder can have multiple api keys configured. An api key can be configured as string ("<key_id>:<key_secret>")
# or as key/value pair ({id = "<key_id>", secret = "<key_secret>"})
api_keys = [{ id = "recorder", secret = "secret" }]

[recorder]
# Shows a display sink, for debug purpose
display = true

# see `man strftime`
# European style - alpine with musl has no locale
clock_format = "%d.%m.%y %X %Z"

# US style
#clock_format = "%x %X %Z"

# Enables Hardware Acceleration for Intel GPUs
#[recorder.hardware_acceleration]
#manufacturer = "intel"
#device = "/dev/dri/renderD129"
```

<!-- end:fromfile:toml:config/example.toml -->
