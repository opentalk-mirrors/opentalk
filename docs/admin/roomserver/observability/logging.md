# Logging

The {{ product_name }} RoomServer prints log messages to `stderr`. The log output can be configured, allowing administrators to control the verbosity and granularity of log messages.

## Configuration

The log level can be configured using the `ROOMSERVER_LOG` environment variable. The log level can be either `OFF`, `ERROR`, `WARN`, `INFO`, `DEBUG` or `TRACE`.
Alternatively the `RUST_LOG` environment variable can be used. `RUST_LOG=<LEVEL>` sets the global log level, affecting the RoomServer and all its dependencies.`RUST_LOG=opentalk=<LEVEL>` only sets the log level of the RoomServer, identical to `ROOMSERVER_LOG`.
When unset, the global log level defaults to `ERROR`, while the RoomServers log level defaults to `INFO`.
