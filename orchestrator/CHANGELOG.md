# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.0] - 2026-09-10

[0.12.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.11.0...v0.12.0

### 🐛 Bug fixes

- (redis) Lower the keep alive interval to two seconds ([!252](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/252), [#49](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/49))
- (redis) Handle OrchestratorServicesKey in remove_instance script ([!252](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/252), [#50](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/50))
- Preserve path in public_url when joining 'roomserver/' ([!253](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/253))
- (docs) Add missing documentation in example.toml ([!253](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/253))
- (recorder) Handle the 204 created response on init call ([!259](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/259))

### 🔨 Refactor

- (livekit-proxy) Add more trace/debug logging ([!248](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/248))

### Ci

- Remove unused code ()

## [0.10.0] - 2026-07-17

[0.10.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.9.0...v0.10.0

### 📦 Dependencies

- (deps) Update opentalk ([!197](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/197))

## [0.9.0] - 2026-07-16

[0.9.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.8.1...v0.9.0

### 📦 Dependencies

- (deps) Update rust crate rustls to v0.23.42 ([!189](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/189))
- (deps) Update opentalk ([!177](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/177))
- (deps) Update rust crate spin ([!177](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/177))

### ⚙ Miscellaneous

- (roomserver) Update roomserver crate to 0.2.0 ([!179](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/179))

## [0.8.0] - 2026-05-21

[0.8.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.7.0...v0.8.0

### 🚀 New features

- (transcription) Integrate the transcription service ([!134](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/134), [#7](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/7))

### 🐛 Bug fixes

- (client) Use event type when sending metrics ([!157](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/157))
- (livekit-proxy) Remove 'v1' version prefix for roomserver livekit endpoints ([!157](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/157))

### 📦 Dependencies

- (deps) Lock file maintenance ([!144](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/144), [!151](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/151), [!156](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/156))
- (deps) Update rust crate tokio to v1.52.2 ([!147](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/147))
- (deps) Update opentalk to 0.0.39 ([!155](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/155))

## [0.7.0] - 2026-04-24

[0.7.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.5.0...v0.7.0

### 🚀 New features

- (orchestrator) Add recorder service backend ([!120](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/120))
- Add livekit proxy endpoints ([!111](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/111), [#26](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/26))
- Warn about unknown configuration fields ([!138](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/138))

### 🔨 Refactor

- (client) Restructure signaling types and add tests ([!130](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/130), [#19](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/19))

### 📦 Dependencies

- (deps) Update rust crate tokio to v1.51.1 ([!119](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/119))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.1 ([!116](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/116))
- (deps) Lock file maintenance ([!124](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/124))
- (deps) Update opentalk ([!125](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/125))
- (deps) Update rust crate axum to v0.8.9 ([!126](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/126))
- (deps) Update rust crate tokio to v1.52.0 ([!128](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/128))
- (deps) Lock file maintenance ([!133](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/133))
- (deps) Update opentalk ([!127](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/127))
- (deps) Update opentalk ([!140](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/140))

## [0.6.0] - 2026-04-22

[0.6.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.5.0...v0.6.0

### 🚀 New features

- (orchestrator) Add recorder service backend ([!120](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/120))
- Add livekit proxy endpoints ([!111](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/111), [#26](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/26))

### 🔨 Refactor

- (client) Restructure signaling types and add tests ([!130](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/130), [#19](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/19))

### 📦 Dependencies

- (deps) Update rust crate tokio to v1.51.1 ([!119](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/119))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.1 ([!116](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/116))
- (deps) Lock file maintenance ([!124](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/124))
- (deps) Update opentalk ([!125](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/125))
- (deps) Update rust crate axum to v0.8.9 ([!126](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/126))
- (deps) Update rust crate tokio to v1.52.0 ([!128](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/128))
- (deps) Lock file maintenance ([!133](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/133))
- (deps) Update opentalk ([!127](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/127))

## [0.5.0] - 2026-04-09

[0.5.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.4.1...v0.5.0

### 🚀 New features

- (orchestrator) Add logging configuration and default directives ([!107](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/107), [#22](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/22))
- Add roomserver signaling proxy ([!100](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/100), [#25](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/25))
- (orchestrator) Add fallback 404 handler ([!108](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/108), [#32](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/32))

### 🐛 Bug fixes

- (logging) Add more extensive debug logging ([!103](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/103))
- Ensure CryptoProvider is configured ([!110](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/110))
- Allow `GET` and `CONNECT` methods to upgrade ws requests ([!114](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/114))
- Use RecorderResource in RecorderEvent ([!121](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/121))

### 🔨 Refactor

- (orchestrator) Replace log crate with tracing ([!107](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/107))

### 📦 Dependencies

- (deps) Update `opentalk-types-common` in `opentalk-orchestrator-shared` only ([!115](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/115))
- (deps) Update all dependencies ([!112](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/112), [!118](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/118))

### ⚙ Miscellaneous

- (toml) Format toml files ([!104](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/104))

### Ci

- (just) Switch changelog tool to opentalk git-cliff ([!109](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/109))
- Use changelog template ([!113](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/113))

## [0.4.1] - 2026-03-17

[0.4.1]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.4.0...v0.4.1

### 🐛 Bug fixes

- Use rwlock instead of mutex ([!102](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/102))

### ⚙ Miscellaneous

- Rename `orchestrator` crate to `opentalk-orchestrator` ([!105](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/105))
- Do not use default-feature for service auth crate ([!105](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/105))

## [0.4.0] - 2026-03-11

[0.4.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.3.0...v0.4.0

### 🚀 New features

- (ci) Add release mr creation job ([!80](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/80), [#20](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/20))
- Add sigterm handler to axum listen ([!87](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/87))
- (orchestrator) Select service instances based on their load metrics ([!92](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/92))
- Add CLI command for orchestrator metrics ([!97](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/97))
- Add endpoint and CLI command for orchestrator service state ([!97](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/97))
- (roomserver) Implement patch_room function ([!99](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/99))
- Allow services to register by only providing a port ([!98](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/98), [#24](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/issues/24))

### 🐛 Bug fixes

- (roomserver) Rename tracked resources from `breakout_rooms` to `rooms` ([!86](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/86))
- (orchestrator) Add test attribute to instance selection tests ([!94](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/94))

### 🔨 Refactor

- Use a JoinSet to manage spawned tasks ([!96](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/96))

### 📦 Dependencies

- (deps) Update rust crate opentalk-types-api-v1 to v0.52.3 ([!79](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/79))
- (deps) Update rust crate clap to v4.5.58 ([!78](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/78))
- (deps) Update opentalk ([!73](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/73), [!81](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/81))
- (deps) Use types-common instead of types with common feature ([!74](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/74))
- (deps) Update rust crate futures-util to v0.3.32 ([!82](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/82))
- (deps) Update rust crate clap to v4.5.59 ([!85](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/85))
- (deps) Update rust crate rand to 0.10 ([!76](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/76))
- (deps) Update rust crate opentalk-types-api-v1 to 0.53 ([!90](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/90))
- (deps) Lock file maintenance ([!77](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/77), [!83](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/83), [!91](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/91), [!93](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/93))
- (deps) Update rust crate tokio to v1.50.0 ([!95](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/95))
- (deps) Update opentalk crates ([!99](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/99))
- (deps) Update roomserver types ([!101](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/101))

## [0.3.0] - 2026-02-05

[0.3.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.2.0...v0.3.0

### 📦 Dependencies

- (deps) Update opentalk-service-auth ([!70](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/70))

## [0.2.0] - 2026-02-04

[0.2.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.1.0...v0.2.0

### 🐛 Bug fixes

- (justfile) Use workspace version path ([!53](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/53))
- (client) Remove enum tagging from service types ([!64](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/64))
- (client) Get mutable access in StateProvider methods ([!68](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/68))

### 📦 Dependencies

- (deps) Lock file maintenance ([!55](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/55), [!57](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/57), [!63](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/63))
- (deps) Update opentalk ([!56](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/56), [!67](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/67), [!59](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/59))
- (deps) Update rust crate clap to v4.5.56 ([!60](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/60))
- (deps) Update rust crate bytes to v1.11.1 ([!65](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/65))
- (deps) Update rust crate clap to v4.5.57 ([!66](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/66))

## [0.1.1] - 2026-01-26

[0.1.1]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/compare/v0.1.0...v0.1.1

### 🐛 Bug fixes

- (justfile) Use workspace version path ([!53](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/53))

### 📦 Dependencies

- (deps) Lock file maintenance ([!55](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/55), [!57](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/57))
- (deps) Update opentalk ([!56](https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/merge_requests/56))

## [0.1.0] - 2026-01-15

[0.1.0]: https://git.opentalk.dev/opentalk/backend/services/orchestrator/-/commits/v0.1.0

The initial `Orchestrator` release!
