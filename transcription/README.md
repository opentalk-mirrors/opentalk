<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# OpenTalk Transcription

The OpenTalk Transcription service joins OpenTalk conference rooms, receives the
participants' audio tracks via LiveKit and turns speech into text using a local
[whisper.cpp](https://github.com/ggml-org/whisper.cpp) model (through the
[`whisper-rs`](https://crates.io/crates/whisper-rs) bindings). The transcribed
segments are sent back to the OpenTalk conference.

## Compute backends

Whisper inference is hardware accelerated. The backend is selected at compile
time, exactly one backend should be enabled.

| Backend     | Cargo feature     | Default |
| ----------- | ----------------- | ------- |
| Vulkan      | `vulkan`          | yes     |
| NVIDIA CUDA | `whisper-rs/cuda` | no      |

Enabling a non-default backend requires disabling the default features, see
[Building](#building).

Pre-built container images are available for both backends, see the
[hardware acceleration guide](docs/admin/hardware_acceleration.md).

## Installation dependencies

### Common build dependencies

Building `whisper-rs` compiles the bundled whisper.cpp C++ sources, so a C/C++
toolchain and CMake are always required.

On Debian/Ubuntu:

```sh
apt-get install build-essential pkg-config clang libclang-dev cmake libssl-dev
```

On Fedora:

```sh
dnf install gcc gcc-c++ make pkgconf-pkg-config clang clang-devel cmake openssl-devel
```

### Vulkan (default)

In addition to the common dependencies, the Vulkan backend needs the Vulkan
loader, headers and the shader compiler.

On Debian/Ubuntu:

```sh
apt-get install libvulkan-dev glslc
```

On Fedora:

```sh
dnf install vulkan-loader-devel vulkan-headers glslc
```

At runtime a Vulkan loader and a driver for your GPU are required.

On Debian/Ubuntu:

```sh
apt-get install libvulkan1 mesa-vulkan-drivers
```

On Fedora:

```sh
dnf install vulkan-loader mesa-vulkan-drivers
```

### NVIDIA CUDA

The CUDA backend requires a working [CUDA toolkit](https://developer.nvidia.com/cuda-downloads)
including `nvcc` and a compatible host compiler:

- `nvcc` must be on the `PATH` (e.g. `/usr/local/cuda/bin`).
- `NVCC_CCBIN` can point to a different host compiler, if the default one is not
  supported by your CUDA version.
- `CMAKE_CUDA_ARCHITECTURES` can be set to build for other GPUs than the ones
  present on the build machine.

At runtime the NVIDIA driver must be installed. The
[CUDA container build](../container/transcription/Dockerfile-cuda) shows a
complete build environment.

### Whisper model

The service loads a whisper.cpp `ggml` model. The default model
`models/ggml-large-v3-turbo.bin` is bundled in this repository.
After checking out the repository the model file must be pulled separately,
via `git lfs pull` command.
A different model can be configured through the `transcription.model` setting, see the
[example configuration](extra/example.toml).

## Building

Build with the default (Vulkan) backend:

```sh
cargo build --release
```

Build with the CUDA backend instead:

```sh
cargo build --release --no-default-features --features whisper-rs/cuda
```

## Configuration

An example configuration is available in the
[`extra/example.toml`](extra/example.toml) file. It can be copied to the root
directory:

```sh
cp ./extra/example.toml ./config.toml
```

or be passed explicitly:

```sh
cargo run -- -c ./extra/example.toml
```

## Administration

See the [administration guide](docs/admin/README.md) for configuration and
operational details.
