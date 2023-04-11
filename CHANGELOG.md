# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fixed incorrect handling of an HTTP response, resulting in a call hang-up when entering an invalid id/pin. ([#28](https://git.opentalk.dev/opentalk/k3k-obelisk/-/issues/28))
- Fixed overlapping of room audio while welcome message is played back. ([#34](https://git.opentalk.dev/opentalk/k3k-obelisk/-/issues/34))
- Fixed a crash when the waiting-room was active while joining. ([#35](https://git.opentalk.dev/opentalk/k3k-obelisk/-/issues/35))

## [0.1.0-rc.2] - 2023-01-31

### Added

- Add license information

## [0.1.0-rc.1] - 2023-01-23

### Added

- Raise and lower hand via DTMF button 2 ([#14](https://git.opentalk.dev/opentalk/k3k-obelisk/-/issues/14))
- Stop playback on DTMF button 0 ([#29](https://git.opentalk.dev/opentalk/k3k-obelisk/-/issues/29))
- Handle moderator muting of participants ([#16](https://git.opentalk.dev/opentalk/k3k-obelisk/-/issues/16))

### Changed

- Update audio files ([#27](https://git.opentalk.dev/opentalk/k3k-obelisk/-/issues/27))

## [0.0.0-internal-release.4] - 2022-12-02

### Added

- implement service authentication using the client_credentials flow

### Fixed

- fixed a bug where environment variables did not overwrite config values

## [0.0.0-internal-release.3] 2022-09-06

### Fixed

- Wait for the publish webrtc-connection to be established before announcing it on the signaling layer, resolving a race-condition where peers would try to subscribe too soon

## [0.0.0-internal-release.2] 2022-06-24

### Fixed

- container: update image to alpine 3.16 providing the latest GStreamer libraries (1.20) which are required for audio-level indication support

## [0.0.0-internal-release.1] 2022-06-23

initial release candidate

[Unreleased]: https://git.opentalk.dev/opentalk/k3k-obelisk/-/compare/v0.1.0-rc.2...main
[0.1.0-rc.2]: https://git.opentalk.dev/opentalk/k3k-obelisk/-/compare/v0.1.0-rc.1...v0.1.0-rc.2
[0.1.0-rc.1]: https://git.opentalk.dev/opentalk/k3k-obelisk/-/compare/a71fc807cdc848b628f1730e44df2ca7d4b11012...v0.1.0-rc.1
[0.0.0-internal-release.4]: https://git.opentalk.dev/opentalk/k3k-obelisk/-/compare/b5d50ad8882992177bbd40a828101bf3837ea8f4...a71fc807cdc848b628f1730e44df2ca7d4b11012
[0.0.0-internal-release.3]: https://git.opentalk.dev/opentalk/k3k-obelisk/-/compare/bb85fa8bac24cd5fcd0a95270452d490bbb372dc...b5d50ad8882992177bbd40a828101bf3837ea8f4
[0.0.0-internal-release.2]: https://git.opentalk.dev/opentalk/k3k-obelisk/-/compare/98a54d32300aaf358f4e0e5cc76a2ee44aa0c10f...bb85fa8bac24cd5fcd0a95270452d490bbb372dc
[0.0.0-internal-release.1]: https://git.opentalk.dev/opentalk/k3k-obelisk/-/commits/98a54d32300aaf358f4e0e5cc76a2ee44aa0c10f
