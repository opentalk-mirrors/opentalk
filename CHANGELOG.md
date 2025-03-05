# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.19.0] - 2025-03-05

[0.19.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.18.3...v0.19.0

### 🚀 New features

- Add cli argument parsing and implement version argument ([!199](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/199))
- Add config argument ([!199](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/199))
- Provide default stun port if its missing from the url ([!208](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/208), [#65](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/65))
- Add TTS tooling and text files ([!211](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/211), [#15](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/15))
- Add additional text messages ([!211](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/211))
- Add english messages ([!211](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/211))
- Send picture-fast-update INFO request on decoder error ([!213](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/213))
- Add sip.encode_video_at_half_bitrate option ([!216](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/216), [#143](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/143))
- Add recording support, add separate video mute binding & add english language support ([!223](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/223))
- Add ubuntu noble based container image ([!229](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/229))
- Add container security scanning ([!238](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/238))

### 🐛 Bug fixes

- Do not decode h264 on the tokio runtime ([!202](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/202))
- Exit gracefully on SIGTERM signal (for shutdown in docker container) ([!208](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/208), [#50](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/50))
- Avoid crash when receiving re-INVITE without SDP body ([!205](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/205), [#142](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/142))
- Improve H.264 compatibility by properly negotiating encoder settings and video resolution ([!204](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/204))
- Play a track when the call-in user is muted by a moderator ([!209](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/209), [#133](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/133))
- Fix video speaker ordering by updating opentalk-compositor to 0.13.1 ([!224](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/224))
- Disable compositor video support when there are no active video streams ([!215](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/215))
- Gracefully handle being moved into the waiting room ([!225](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/225))
- Reduce container size and attack surface ([!229](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/229))
- Set KANIKO_IMAGE variable in .gitlab CI ([!233](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/233))
- (ci) Temporary ignore RUSTSEC-2025-0008 ([!233](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/233))
- Handle livekit microphone restrictions ([!226](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/226))

### 📚 Documentation

- Fix broken link ([!200](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/200))

### 🔨 Refactor

- (ci) Clean up install step in Dockerfiles ([!229](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/229))

### 📦 Dependencies

- (deps) Update rust crate config to 0.15 ([!198](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/198))
- (deps) Update ezk-rs, fixes RUSTSEC-2024-0421 ([!193](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/193))
- (deps) Update opentalk to 0.29.0 ([!203](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/203))
- (deps) Lock file maintenance ([!203](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/203))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.84.0 ([!207](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/207))
- (deps) Update opentalk to 0.30.1 ([!210](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/210))
- (deps) Update rust crate service-probe to v0.2.1 ([!220](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/220))
- (deps) Update rust crate openh264 to 0.8 ([!232](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/232))
- (deps) Update opentalk to 0.32.0 ([!237](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/237))

### ⚙ Miscellaneous

- Update dependencies ([!202](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/202))
- Update openidconnect, reqwest (http) and tokio-tungstenite (websocket) dependencies ([!214](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/214))
- Update opentalk-types, ezk, rand & livekit ([!222](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/222))
- (ci) Update rust container image to 1.85 ([!236](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/236))
- Lock file maintenance ([!232](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/232))
- Document resource requirements ([!239](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/239))

### Ci

- Verify that commits are signed ([!201](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/201))

## [0.18.0] - 2024-12-12

[0.18.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.17.0...v0.18.0

### 🚀 New features

- Support media sessions via ipv6 ([!188](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/188))

### 🐛 Bug fixes

- Emit video at the caller's requested framerate ([!189](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/189))

### 📚 Documentation

- (monitoring) Explain behaviour when monitoring is not configured ([!187](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/187))

### 📦 Dependencies

- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.83.0 ([!191](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/191))
- (deps) Update rust crate opentalk-compositor to 0.11 ([!195](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/195))
- (deps) Update rust crate service-probe to 0.2 ([!194](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/194))
- (deps) Update rust opentalk crates to 0.28.0 ([!195](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/195))

### ⚙ Miscellaneous

- Update dependencies ([!189](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/189))
- Create changelog with just ([!182](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/182))
- Ignore RUSTSEC-2024-0421 ([!194](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/194))
- (just) Separate checks in justfile ([!197](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/197))

### Ci

- Group opentalk crates ([!195](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/195))

## [0.17.0] - 2024-11-21

[0.17.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.16.0...v0.17.0

### 🚀 New features

- Add an endpoint to determine the readiness of the service ([!183](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/183))

### 📦 Dependencies

- (deps) Update rust crate types to 0.27.0 ([!162](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/162))
- (deps) Update rust crate `service-probe` to version 0.1.1 ([!162](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/162))

## [0.16.0]

[0.16.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.15.0...v0.16.0

### 🚀 New features

- Re-add H.264 video support ([!177](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/177))
- Re-add G.722 audio codec support ([!179](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/179))

### 📦 Dependencies

- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.82.0 ([!174](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/174))
- (deps) Update rust crate thiserror to v2 ([!176](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/176))

## [0.15.0]

[0.15.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.11.0...v0.15.0

### 🚀 New features

- Check if the host system supports avx ([!136](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/136))

### 🐛 Bug fixes

- Only subscribe screenshare when call uses video ([!166](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/1166))

### 🔨 Refactor

- Rewrite media pipeline using livekit & ezk, removing GStreamer ([!171](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/171))

### 📦 Dependencies

- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.82.0 ([!164](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/164))

### ⚙ Miscellaneous

- Migrate to new reuse config format ([!165](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/165))

### Ci

- Introduce changelog bot ([!165](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/165))
- Enforce toml formatting ([!167](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/167))
- (fix) Don't lint commits on main ([!170](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/170))

## [0.11.1]

[0.11.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.11.0...0.11.1

### 🐛 Bug fixes

- Only subscribe screenshare when call uses video ([!166](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/166))

## [0.11.0]

[0.11.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.10.0...0.11.0

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

[0.10.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.10.0...v0.10.1

### 🐛 Bug fixes

- negotiate valid DTMF digit range ([#119](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/119))

### 📦 Dependencies

- update openssl rust crate to 0.10.66 (fixes `RUSTSEC-2024-0357`)

## [0.10.0]

[0.10.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.9.0...v0.10.0

### 🚀 New features

- Show screen shares in video calls ([!121](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/121))

### 🐛 Bug fixes

- Use compositor's add_watch_to_sink instead of registering own watcher ([!143](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/143))

### ⚙ Miscellaneous

- Update gstreamer to 0.23

## [0.9.1]

[0.9.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.9.0...v0.9.1

### 🐛 Bug fixes

- negotiate valid DTMF digit range ([#119](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/119))

### 📦 Dependencies

- update openssl rust crate to 0.10.66 (fixes `RUSTSEC-2024-0357`)
- update bytes rust crate 1.7.1 (1.6.0 has been yanked from crates.io)

## [0.9.0]

[0.9.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.8.0...v0.9.0

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

[0.8.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.7.0...v0.8.0

### Added

- Support for video calls via H.264 ([#64](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/64))

### Fixed

- Use generated contact instead of id in invite sessions ([#76](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/76))
- Revert mute by default and redo usage message ([#74](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/74))
- Add openh264 dependency ([#87](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/87))

## [0.7.2]

[0.7.2]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.7.1...v0.7.2

### Fixed

- RUSTSEC-2024-0019 by updating mio to 0.8.11
- RUSTSEC-2024-0332 by updating h2 to 0.3.26
- RUSTSEC-2024-0336 by updating rustls to 0.21.11

## [0.7.1]

[0.7.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.7.0...v0.7.1

### Fixed

- Use generated contact instead of id in invite sessions ([#75](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/75))
- Revert mute by default and redo usage message ([#74](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/74))

## [0.7.0]

[0.7.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.5.0...v0.7.0

### Changed

- Update build and base container image
- Update dependencies

### Added

- Add `sip.contact` configuration to allow overriding the contact header in SIP messages
- Mute dial-in participants by default ([#67](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/67))

### Fixed

- Fix `sip.id` config to only apply to REGISTER message's From and To header. Previously this also affected the contact header of SIP messages which could cause unwanted behavior.

## [0.5.0] - 2023-10-30

[0.5.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.4.0...v0.5.0

### Added

- Add support for SIP over TCP or TLS ([!75](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/75))
- Add NAPTR & SRV service discovery for SIP registrars ([#52](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/52))
- Add setting `sip.outbound_proxy` to specify a SIP proxy to send registration requests to (instead of `sip.registrar`) ([!75](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/75))

### Fixed

- Fixed the issue where the "welcome to opentalk" message was cut off at the beginning when establishing a new SIP Call with the obelisk ([#43](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/43))
- Handle 423 error responses from SIP registration requests properly ([!75](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/merge_requests/75))

## [0.4.0] - 2023-08-24

[0.4.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.3.0...v0.4.0

### Added

- Add support for ICE full-trickle mode

### Fixed

- Upgrade all dependencies plus cargo update
- Add CA utils to container to be able to add custom certificates
- Add support for platform based certificate verification
- Properly handle failing webrtc subscriptions which never produced any data and caused the media pipeline to pause ([#26](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/26))
- Fix leaking media elements, which caused exhaustion of memory and open files over time, crashing the application ([#45](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/45))

## [0.3.0] - 2023-06-27

[0.3.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.2.0...v0.3.0

### Added

- Added a config `sip.id` to allow overriding the obelisk's SIP ID

### Fixed

- Offer old signaling protocol to enable backwards compatibility with older controller versions
- Miscellaneous internal bugfixes and stability improvements
- Increase the delay of the welcome message ([#42](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/42))

## [0.2.1] - 2023-05-30

[0.2.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.2.0...v0.2.1

- Added a config `sip.id` to allow overriding the obelisk's SIP ID

### Added

- Added a config `sip.id` to allow overriding the obelisk's SIP ID

## [0.2.0] - 2023-04-17

[0.2.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.1.0...v0.2.0

### Added

- Added an announcement at the end of a conference. ([#36](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/36))

### Fixed

- Fixed incorrect handling of an HTTP response, resulting in a call hang-up when entering an invalid id/pin. ([#28](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/28))
- Fixed overlapping of room audio while welcome message is played back. ([#34](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/34))
- Fixed a crash when the waiting-room was active while joining. ([#35](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/35))

## [0.1.1] - 2023-03-21

[0.1.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/v0.1.0...v0.1.1

### Fixed

- Fixed overlapping of room audio while welcome message is played back. ([#34](https://git.opentalk.dev/opentalk/backend/services/obelisk/-/issues/34))

## [0.1.0] - 2023-03-01

[0.1.0]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/0.0.0-internal-release.4...v0.1.0

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

[0.0.0-internal-release.4]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/b5d50ad8882992177bbd40a828101bf3837ea8f4...a71fc807cdc848b628f1730e44df2ca7d4b11012

### Added

- implement service authentication using the client_credentials flow

### Fixed

- fixed a bug where environment variables did not overwrite config values

## [0.0.0-internal-release.3] 2022-09-06

[0.0.0-internal-release.3]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/bb85fa8bac24cd5fcd0a95270452d490bbb372dc...b5d50ad8882992177bbd40a828101bf3837ea8f4

### Fixed

- Wait for the publish webrtc-connection to be established before announcing it on the signaling layer, resolving a race-condition where peers would try to subscribe too soon

## [0.0.0-internal-release.2] 2022-06-24

[0.0.0-internal-release.2]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/compare/98a54d32300aaf358f4e0e5cc76a2ee44aa0c10f...bb85fa8bac24cd5fcd0a95270452d490bbb372dc

### Fixed

- container: update image to alpine 3.16 providing the latest GStreamer libraries (1.20) which are required for audio-level indication support

## [0.0.0-internal-release.1] 2022-06-23

[0.0.0-internal-release.1]: https://git.opentalk.dev/opentalk/backend/services/obelisk/-/commits/98a54d32300aaf358f4e0e5cc76a2ee44aa0c10f

initial release candidate
