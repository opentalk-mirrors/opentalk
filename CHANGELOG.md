Changelog
---
## 1.0.0-rc.3 (06 September, 2022)

### Fixed

- Wait for the publish webrtc-connection to be established before announcing it on the signaling layer, resolving a race-condition where peers would try to subscribe too soon

## 1.0.0-rc.2 (24 June, 2022)

### Fixed

- container: update image to alpine 3.16 providing the latest GStreamer libraries (1.20) which are required for audio-level indication support

## 1.0.0-rc.1 (23 June, 2022)

initial release candidate
