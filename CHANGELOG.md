# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.11.1]

### 🐛 Bug fixes

- Only subscribe screenshare when call uses video ([!166](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/166))

## [0.11.0]

### 🚀 New features

- Offer SDP on empty INVITEs ([#102](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/102), [!144](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/144))

### 🐛 Bug fixes

- Use compositor's add_watch_to_sink instead of registering own watcher ([!143](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/143))
- Use DTMF's fmtp field from SDP offer when creating SDP response ([!146](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/146))
- Only subscribe video with video pipeline ([!156](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/156))
- Remove to-tag from register messages ([!155](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/155))

### 📦 Dependencies

- (deps) Update openssl to 0.10.66 ([!149](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/149))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.81.0 ([!152](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/152))
- (deps) Update rust crates futures-* to 0.3.31 ([!160](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/160))
- (deps) Update opentalk-compositor and opentalk-types ([!160](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/160))

### ⚙ Miscellaneous

- Add documentation for supported codecs ([!150](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/150))
- Ignore RUSTSEC-2024-0370 ([!151](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/151))
- Update compositor submodule ([!156](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/156))
- Run cargo update ([!156](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/156))
- (release) Add a `justfile` with a `create-release` target for release automation ([!159](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/159))

## [0.10.1]

### 🐛 Bug fixes

- negotiate valid DTMF digit range ([#119](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/119))

### 📦 Dependencies

- update openssl rust crate to 0.10.66 (fixes `RUSTSEC-2024-0357`)

## [0.9.1]

### 🐛 Bug fixes

- negotiate valid DTMF digit range ([#119](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/119))

### 📦 Dependencies

- update openssl rust crate to 0.10.66 (fixes `RUSTSEC-2024-0357`)
- update bytes rust crate 1.7.1 (1.6.0 has been yanked from crates.io)

## [0.9.0]

### 🚀 New features

- Re-add watchdog watching RTP traffic to detect interruptions and terminate calls
- Add SRTP support for calls ([#66](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/66))
- Add a timeout to the pin entry ([#92](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/92))

### 🐛 Bug fixes

- *(srtp)* Add padding to base64 encoding of key
- Handle SIP events (like hangup) while in waiting room ([#93](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/93))

### ⚙ Miscellaneous Tasks

- Change base image to alpine3.20

### Ci

- Prevent ci pipeline running on fork
- Use image with fixed rust version
- Call `cargo-deny` with `--deny unmatched-skip --deny license-not-encountered` ([#104](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/104))


## [0.8.0]

### Added

- Support for video calls via H.264 ([#64](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/64))

### Fixed

- Use generated contact instead of id in invite sessions ([#76](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/76))
- Revert mute by default and redo usage message ([#74](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/74))
- Add openh264 dependency ([#87](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/87))

## [0.7.2]

### Fixed

- RUSTSEC-2024-0019 by updating mio to 0.8.11
- RUSTSEC-2024-0332 by updating h2 to 0.3.26
- RUSTSEC-2024-0336 by updating rustls to 0.21.11

## [0.7.1]

### Fixed

- Use generated contact instead of id in invite sessions ([#75](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/75))
- Revert mute by default and redo usage message ([#74](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/74))

## [0.7.0]

### Changed

- Update build and base container image
- Update dependencies

### Added

- Add `sip.contact` configuration to allow overriding the contact header in SIP messages
- Mute dial-in participants by default ([#67](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/67))

### Fixed

- Fix `sip.id` config to only apply to REGISTER message's From and To header. Previously this also affected the contact header of SIP messages which could cause unwanted behavior.

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

[0.11.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.11.0...0.11.1
[0.11.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.10.0...0.11.0

[0.10.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.10.0...v0.10.1
[0.10.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.9.0...v0.10.0

[0.9.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.9.0...v0.9.1
[0.9.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.8.0...v0.9.0

[0.8.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.7.0...v0.8.0

[0.7.2]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.7.1...v0.7.2
[0.7.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.7.0...v0.7.1
[0.7.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.5.0...v0.7.0

[0.5.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.4.0...v0.5.0

[0.4.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.3.0...v0.4.0

[0.3.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.2.0...v0.3.0

[0.2.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.2.0...v0.2.1
[0.2.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.1.0...v0.2.0

[0.1.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.1.0...v0.1.1
[0.1.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/0.0.0-internal-release.4...v0.1.0

[0.0.0-internal-release.4]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/b5d50ad8882992177bbd40a828101bf3837ea8f4...a71fc807cdc848b628f1730e44df2ca7d4b11012
[0.0.0-internal-release.3]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/bb85fa8bac24cd5fcd0a95270452d490bbb372dc...b5d50ad8882992177bbd40a828101bf3837ea8f4
[0.0.0-internal-release.2]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/98a54d32300aaf358f4e0e5cc76a2ee44aa0c10f...bb85fa8bac24cd5fcd0a95270452d490bbb372dc
[0.0.0-internal-release.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/commits/98a54d32300aaf358f4e0e5cc76a2ee44aa0c10f
