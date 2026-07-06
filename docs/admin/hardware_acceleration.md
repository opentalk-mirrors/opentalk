# Hardware Acceleration

The Transcription service runs whisper.cpp inference on the
GPU. Hardware acceleration significantly increases the number of audio streams a
single instance can transcribe in real time compared to CPU-only execution.

The backend is selected at compile time through a Cargo feature; exactly one
backend should be enabled. The build and runtime dependencies of each backend
are described in the `README.md` at the root of the repository.

## Vulkan

Vulkan is the default backend and runs on a wide range of GPUs. It is enabled
through the `vulkan` Cargo feature (enabled by default). At runtime a Vulkan
loader and a driver for the GPU must be available.

## NVIDIA CUDA

The CUDA backend is enabled through the `cuda` Cargo feature and requires a
working CUDA toolkit at build time and matching NVIDIA drivers at runtime. The
`run-cuda` helper script in the repository shows the expected build environment.

## Intel oneAPI (SYCL)

The SYCL backend is enabled through the `sycl` Cargo feature and builds against
the Intel oneAPI Base Toolkit. The oneAPI environment must be sourced before
building and running, as shown in the `run-intel` helper script.

## Without hardware acceleration

Running whisper.cpp on the CPU is possible but considerably slower and does not
scale to many concurrent streams. Hardware acceleration is strongly recommended
for production deployments.
