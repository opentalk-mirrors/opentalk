<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# OpenTalk Recorder

See the [administration guide](docs/admin/README.md) for more information.

## Configuration

See the [configuration](docs/admin/configuration.md) chapter of the
administration guide for more information.

An example configuration is available in the [`extra/example.toml`](extra/
example.toml) file. It can be copied to the root directory:

```sh
cp ./extra/example.toml ./config.toml
```

or be used with

```sh
cargo run -- -c ./extra/example.toml
```

## Limitations

### File Descriptor limitations

The OpenTalk Recorder utilizes [GStreamer](https://gstreamer.freedesktop.org/)
as its underlying system, and GStreamer relies heavily on file descriptors
for its operations. By default, the ulimit for file descriptors is often set
to 1024. Each participant in the recording process requires approximately 64
file descriptors, meaning that the recording can accommodate only up to 16
participants under this default limit. You can check the current ulimit for
file descriptors using the command `ulimit -n`. To overcome this limitation, you
have the option to increase the ulimit using `ulimit -n 10000` or set it to an
unlimited value with `ulimit -n unlimited`. We highly recommend increasing the
ulimit for optimal performance.

It's also worth noting that you can adjust the ulimit within the [docker-
compose](https://docs.docker.com/compose/compose-file/compose-file-v3/#ulimits)
configuration if you are using Docker.

## Build the container image

The `Dockerfile` is located at `ci/Dockerfile`.

To build the image, execute in the root of the repository:

```bash
docker build -f ci/Dockerfile . --tag <your tag>
```
