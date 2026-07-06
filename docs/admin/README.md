---
title: Transcription
---

# Administration guide for the Transcription service

The Transcription service joins conference rooms, receives
the participants' audio via LiveKit and turns their speech into text using a
local [whisper.cpp](https://github.com/ggml-org/whisper.cpp) model. The
resulting transcript segments are sent back to the conference.

## General information about the service

- [Configuration](./configuration/README.md)
- [Hardware Acceleration](./hardware_acceleration.md)
- [HTTP](./configuration/http.md)
- [Monitoring](./configuration/monitoring.md)

### Environment

- `TRANSCRIPTION_LOG` to configure the log filter. It takes precedence over
  `RUST_LOG`. When neither is set, the default
  `warn,opentalk_transcription=info,whisper_rs=warn` is used.

#### Debugging and development

- `RUST_BACKTRACE: full` to get detailed stack traces.
- `RUST_LOG: debug,opentalk_transcription=trace` for more verbose logging.

## Interaction between Transcription and other services

### Services required by Transcription

- [Controller](./configuration/controller.md)

### Services that can use Transcription

- [Orchestrator](./configuration/orchestrator.md)
