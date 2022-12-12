Changelog
---

## Unreleased

### Added

- Raise and lower hand via DTMF button 2 ([#14](https://git.opentalk.dev/heinlein-video/k3k-obelisk/-/issues/14))
- Stop playback on DTMF button 0 ([#29](https://git.opentalk.dev/heinlein-video/k3k-obelisk/-/issues/29))
- Handle moderator muting of participants ([#16](https://git.opentalk.dev/opentalk/k3k-obelisk/-/issues/16))

### Changed

- Update audio files ([#27](https://git.opentalk.dev/heinlein-video/k3k-obelisk/-/issues/27))

## 1.0.0-rc.4 (02 December, 2022)

### Added

- implement service authentication using the client_credentials flow

### Fixed

- fixed a bug where environment variables did not overwrite config values

## 1.0.0-rc.3 (06 September, 2022)

### Fixed

- Wait for the publish webrtc-connection to be established before announcing it on the signaling layer, resolving a race-condition where peers would try to subscribe too soon

## 1.0.0-rc.2 (24 June, 2022)

### Fixed

- container: update image to alpine 3.16 providing the latest GStreamer libraries (1.20) which are required for audio-level indication support

## 1.0.0-rc.1 (23 June, 2022)

initial release candidate
