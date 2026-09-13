---
title: Recorder
---

# Administration guide for the {{ product_name }} Recorder

## General information about the service

- [Configuration](./configuration/README.md)
- [Migration Guide](./migration.md)
- [Hardware Acceleration](./hardware_acceleration.md)
- [HTTP](./configuration/http.md)
- [Monitoring](./configuration/monitoring.md)
- [Recorder](./configuration/recorder.md)

### Environment

- `TZ: Europe/Berlin` to specify the timezone, see Alpine Linux documentation.

#### Debugging and Development

- `RUST_BACKTRACE: full` to get detailed stack traces
- `RUST_LOG: info,opentalk_recorder=debug,rustls=info,opentalk_recorder::signaling=debug` for more verbose logging
- `GST_DEBUG: 2` to increase GStreamer logging maximum is 6 but 3 is already very noisy
- `GST_DEBUG_FILE: /var/recording_logs/recorder_debug.log` dedicated log file. Necessary for `GST_DEBUG > 3` in combination with a dedicated docker volume
- `GST_DEBUG_DUMP_DOT_DIR: /var/recording_pipelines` dedicated directory to store pipeline snapshots.  Active when `GST_DEBUG > 2` -- we recommend to a dedicated docker volume.

## Interaction between {{ product_name }} Recorder and other services

### Services required by {{ product_name }} Recorder

- [Controller](./configuration/controller.md)

### Optional services

- [Orchestrator](./configuration/orchestrator.md)
