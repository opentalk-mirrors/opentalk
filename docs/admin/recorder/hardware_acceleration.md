# Hardware Acceleration

The `Recorder` is capable of doing Hardware acceleration for the Video
Encoding. Hardware acceleration in video encoding is crucial for achieving
higher efficiency, as GPUs are much more capable than CPUs in handling
parallel processing. By offloading the encoding workload to the GPU, the system
can manage more simultaneous video streams with lower CPU usage and energy
consumption.

## AMD

At this time, AMD GPUs are not supported for hardware acceleration in the
`Recorder`.

## Intel

Intel GPUs, specifically the Intel Arc A310 Eco and Intel Arc A770 models,
have been tested for hardware acceleration. Both GPUs offer the same encoding
capacity since they use identical encoding chips. For `H.264` encoding, these
GPUs can handle up to `46 simultaneous video` streams at `1080p` resolution at
`25` frames per second. When it comes to `AV1` encoding, they can process up to
`14` simultaneous streams.

### Adjusting the `perf_event_paranoid` Kernel Setting

Depending on your operating system and kernel configuration, it may be necessary to modify the `/proc/sys/kernel/perf_event_paranoid` setting.

#### Temporary Change

To test the adjustment, run the following command. This change will last until the next reboot:

```bash
sudo sh -c 'echo 2 >/proc/sys/kernel/perf_event_paranoid'
```

#### Permanent Change

To make the change persistent across reboots, append the setting to your sysctl configuration:

```bash
sudo sh -c 'echo kernel.perf_event_paranoid=2 >> /etc/sysctl.d/local.conf'
```

## Nvidia

Nvidia consumer GPUs, though tested, are limited to encoding a maximum of
8 simultaneous video streams. Unfortunately, Nvidia GPUs are currently not
supported for hardware acceleration in the `Recorder`.

## Without Hardware Acceleration (CPU Encoding)

When hardware acceleration is not available, the encoding task falls back to the
CPU. For `VP8` encoding (which is currently used for `Recording`, `H.264` is used for `Streaming`), `4 CPU cores` are required per video stream, which
significantly limits scalability. Encoding `AV1` on the CPU is not feasible
due to the high processing demands, making it an impractical solution for
high-volume `AV1` encoding without hardware support.
