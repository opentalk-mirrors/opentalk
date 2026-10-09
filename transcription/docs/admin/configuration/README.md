# Configuration

When the Transcription service starts, it loads its
configuration from a configuration file and from environment variables.

## Configuration file discovery

If a path is passed with the `-c`/`--config` command line argument, exactly that
file is used. Otherwise the service looks for a configuration file in these
locations and uses the first one that exists:

- `config.toml` in the current directory (deprecated, for backwards
  compatibility only)
- `transcription.toml` in the current directory
- `<XDG_CONFIG_HOME>/opentalk/transcription.toml` (where `XDG_CONFIG_HOME` is
  usually `~/.config`)
- `/etc/opentalk/transcription.toml`

## Sections in the configuration file

Functionality that can be configured through the configuration file:

- [Auth](auth.md)
- [Controller](controller.md)
- [HTTP](http.md)
- [Monitoring](monitoring.md)
- [Orchestrator](orchestrator.md)
- [Tracing](tracing.md)
- [Transcription](transcription.md)

## Environment variables

Settings in the configuration file can be overwritten by environment variables.
Each variable is prefixed with `OPENTALK_TRANSCRIPTION_`, and nested fields are
separated by two underscores `__`. The pattern looks like this:

```sh
OPENTALK_TRANSCRIPTION_<field>__<nested-field>…
```

### Limitations

Some settings can not be overwritten by environment variables. This is for
example the case for entries in lists, because there is no environment variable
naming pattern that could identify the index of the entry inside the list.

### Examples

In order to set the `http.port` field, this environment variable could be used:

```sh
OPENTALK_TRANSCRIPTION_HTTP__PORT=11413
```

## Example configuration file

A full, annotated example configuration is available in the source code
distribution under `extra/example.toml`.
