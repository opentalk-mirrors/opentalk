---
title: Obelisk
---

# Administration guide for the {{ product_name }} Obelisk SIP bridge

## General information about the service

- [Configuration](configuration.md)
- [Health command](health.md) for fetching the current readiness state.⏎

## Interaction between {{ product_name }} Obelisk and other services

### Services required by {{ product_name }} Obelisk

- [Auth](auth.md)
- [Controller](controller.md)
- [Monitoring](monitoring.md)
- [SIP Provider](sip.md)

## Resource Requirements

To ensure maximum compatibility with SIP-enabled devices, Obelisk composites the
conference into a single audio and video stream. This process involves decoding
the video of up to nine participants and compositing them into a single video
stream, which is then encoded and sent to the callee.

Currently all encoding and decoding of video streams is done in software
requiring a rather modern CPU.

### Requirements per audio-only participant

On an `Intel i9-14900K` system, Obelisk used `0.2% CPU load` and about `25 MiB`
of system memory for a call with a single SIP devices to a conference with 5
participants talking at the same time.

### Requirements per video participant

On an `Intel i9-14900K` system, Obelisk used `6% CPU load` and `400 MiB` of
system memory for a call with a single SIP device (sending and receiving video
at 1080p@30FPS) to a conference with 9 other participants who had their camera
enabled.
