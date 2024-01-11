# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Mute dial-in participants by default ([#67](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/67))

## [0.5.0] - 2023-10-30

### Added

- Add support for SIP over TCP or TLS ([!75](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/75))
- Add NAPTR & SRV service discovery for SIP registrars ([#52](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/52))
- Add setting `sip.outbound_proxy` to specify a SIP proxy to send registration requests to (instead of `sip.registrar`) ([!75](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/75))

### Fixed

- Fixed the issue where the "welcome to opentalk" message was cut off at the beginning when establishing a new SIP Call with the obelisk ([#43](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/43))
- Handle 423 error responses from SIP registration requests properly ([!75](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/75))

## [0.4.0] - 2023-08-24

### Added

- Add support for ICE full-trickle mode

### Fixed

- Upgrade all dependencies plus cargo update
- Add CA utils to container to be able to add custom certificates
- Add support for platform based certificate verification
- Properly handle failing webrtc subscriptions which never produced any data and caused the media pipeline to pause ([#26](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/26))
- Fix leaking media elements, which caused exhaustion of memory and open files over time, crashing the application ([#45](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/45))

## [0.3.0] - 2023-06-27

### Added

- Added a config `sip.id` to allow overriding the obelisk's SIP ID

### Fixed

- Offer old signaling protocol to enable backwards compatibility with older controller versions
- Miscellaneous internal bugfixes and stability improvements
- Increase the delay of the welcome message ([#42](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/42))

## [0.2.1] - 2023-05-30

- Added a config `sip.id` to allow overriding the obelisk's SIP ID

### Added

- Added a config `sip.id` to allow overriding the obelisk's SIP ID

## [0.2.0] - 2023-04-17

### Added

- Added an announcement at the end of a conference. ([#36](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/36))

### Fixed

- Fixed incorrect handling of an HTTP response, resulting in a call hang-up when entering an invalid id/pin. ([#28](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/28))
- Fixed overlapping of room audio while welcome message is played back. ([#34](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/34))
- Fixed a crash when the waiting-room was active while joining. ([#35](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/35))

## [0.1.1] - 2023-03-21

### Fixed

- Fixed overlapping of room audio while welcome message is played back. ([#34](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/34))

## [0.1.0] - 2023-03-01

### Added

- Raise and lower hand via DTMF button 2 ([#14](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/14))
- Stop playback on DTMF button 0 ([#29](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/29))
- Handle moderator muting of participants ([#16](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/16))
- Add license information

### Changed

- Update audio files ([#27](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/27))

### Fixed

- Fixed incorrect handling of an HTTP response, resulting in a call hang-up when entering an invalid id/pin. ([#28](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/28))

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

[Unreleased]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.5.0...main

[0.5.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/ed499459ca0e207fcb0c915c04c0f99e1cb7c2db...v0.5.0

[0.4.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/ae71d32c12944bff87e2db2955e9bac73451ea37...v0.4.0

[0.3.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/682f4a0eff7048adc2a580b380d4cf1bb9e63096...v0.3.0

[0.2.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.2.0...v0.2.1
[0.2.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/f0b6143249961167f4a5c475a868c73bff20d5e2...v0.2.0

[0.1.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.1.0...v0.1.1
[0.1.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/a71fc807cdc848b628f1730e44df2ca7d4b11012...v0.1.0

[0.0.0-internal-release.4]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/b5d50ad8882992177bbd40a828101bf3837ea8f4...a71fc807cdc848b628f1730e44df2ca7d4b11012
[0.0.0-internal-release.3]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/bb85fa8bac24cd5fcd0a95270452d490bbb372dc...b5d50ad8882992177bbd40a828101bf3837ea8f4
[0.0.0-internal-release.2]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/98a54d32300aaf358f4e0e5cc76a2ee44aa0c10f...bb85fa8bac24cd5fcd0a95270452d490bbb372dc
[0.0.0-internal-release.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/commits/98a54d32300aaf358f4e0e5cc76a2ee44aa0c10f
