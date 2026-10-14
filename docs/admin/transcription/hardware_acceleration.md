# Hardware Acceleration

The Transcription service runs whisper.cpp inference on the GPU. Hardware
acceleration significantly increases the number of audio streams a single
instance can transcribe in real time compared to CPU-only execution.

## Container images

The Transcription service is published as two container images, one per GPU
backend. The backend is part of the image tag as a suffix:

| Image flavor | Tag suffix | GPUs                                                 |
| ------------ | ---------- | ---------------------------------------------------- |
| Vulkan       | `-vulkan`  | GPUs supported by the Mesa Vulkan drivers            |
| NVIDIA CUDA  | `-cuda`    | NVIDIA GPUs of the Turing generation (2018) or newer |

Vulkan is the default flavor, tags without a suffix refer to the Vulkan image.

The images are built for the `x86-64-v3` CPU level, the host CPU must support
AVX2, FMA, F16C and BMI2 (Intel Haswell, AMD Zen or newer).

Both images ship the `ggml-large-v3-turbo.bin` model at the default `model`
path, see the [Transcription configuration](./configuration/transcription.md).

## Vulkan

The Vulkan image contains the Vulkan loader and the Mesa Vulkan drivers. The
GPU's render node must be passed into the container. The service runs as user
`1000:1000`, so it additionally needs to be a member of the group owning the
render node on the host (usually `render`):

```yaml
services:
  transcription:
    image: registry.heinlein.group/opentalk/transcription:<version>-vulkan
    # ...
    devices:
      - /dev/dri
    group_add:
      # GID of the `render` group on the host, see `getent group render`
      - "<render-gid>"
```

## NVIDIA CUDA

The GPU must be reserved for the container:

```yaml
services:
  transcription:
    image: registry.heinlein.group/opentalk/transcription:<version>-cuda
    # ...
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: all
              capabilities: [gpu]
```

## Without hardware acceleration

Running whisper.cpp on the CPU is possible but considerably slower and does not
scale to many concurrent streams. Hardware acceleration is strongly recommended
for production deployments.
