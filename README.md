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
time through a Cargo feature. Exactly one backend should be enabled.

| Backend         | Cargo feature      | Default |
| --------------- | ------------------ | ------- |
| Vulkan          | `vulkan`           | yes     |
| NVIDIA CUDA     | `cuda`             | no      |
| Intel oneAPI    | `sycl`             | no      |

The default build uses the Vulkan backend, which runs on a wide range of GPUs.

## Installation dependencies

### Common build dependencies

Building `whisper-rs` compiles the bundled whisper.cpp C++ sources, so a C/C++
toolchain and CMake are always required. On Debian/Ubuntu:

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

At runtime a Vulkan loader and a driver for your GPU are required, for example.

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
including `nvcc` and a compatible host compiler. The
[`run-cuda`](run-cuda) helper script shows the environment expected by the
build, most importantly:

- `nvcc` must be on the `PATH` (e.g. `/usr/local/cuda/bin`).
- `CMAKE_CUDA_ARCHITECTURES` must match your GPU's compute capability.
- `NVCC_CCBIN` must point to a host compiler supported by your CUDA version.

### Intel oneAPI (SYCL)

The SYCL backend builds against the
[Intel oneAPI Base Toolkit](https://www.intel.com/content/www/us/en/developer/tools/oneapi/base-toolkit.html)
(providing the `icx`/`icpx` compilers and MKL). On Debian/Ubuntu the build
additionally needs:

```sh
apt-get install libomp-dev intel-oneapi-mkl-devel intel-oneapi-mkl clinfo \
    intel-media-va-driver intel-gpu-tools
```

On Fedora:

```sh
dnf install libomp-devel intel-oneapi-mkl-devel intel-oneapi-mkl clinfo \
    intel-media-driver intel-gpu-tools
```

The oneAPI environment must be sourced before building or running, see the
[`run-intel`](run-intel) helper script:

```sh
source /opt/intel/oneapi/setvars.sh
```

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

Build with a specific backend instead:

```sh
cargo build --release --no-default-features --features cuda
cargo build --release --no-default-features --features sycl
```

The [`run-cuda`](run-cuda) and [`run-intel`](run-intel) scripts wrap
`cargo run` with the environment required by the respective backend.

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

## Build the container image

The Dockerfiles are located in the `ci` directory, one per backend. To build an
image, execute in the root of the repository:

```sh
docker build -f ci/Dockerfile-vulkan . --tag <your tag>
```

## Administration

See the [administration guide](docs/admin/README.md) for configuration and
operational details.
