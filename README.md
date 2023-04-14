# OpenTalk Obelisk

SIP bridge for the OpenTalk conference system.

## Build the container image

The `Dockerfile` is located in `ci/Dockerfile`.

To build the image, execute in the root of the repository:

```bash
 docker build -f ci/Dockerfile . --tag <your tag>
```

## GStreamer Pipeline (greatly simplified)

```mermaid
graph LR
  subgraph pipeline
    subgraph sip_bin

      sip-audio-input --> rtp-session
      rtp-session --> sip-audio-output
    end
    
    sip-audio-output --> webrtc-publish

    track-player --> subscriber-audiomix
    subscriber-audiomix --> sip-audio-input

    webrtc-subscriber:PARTICIPANT_ID --> subscriber-audiomix
    
    subgraph n-subscribers
        webrtc-subscriber:PARTICIPANT_ID
    end
  end

  janus-gateway --> |RTP/RTCP| webrtc-subscriber:PARTICIPANT_ID
  webrtc-publish --> |RTP/RTCP| janus-gateway

  phone .- |RTP/RTCP| rtp-session

```
