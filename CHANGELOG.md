# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-07-17

[0.3.0]: https://git.opentalk.dev/opentalk/backend/services/transcription/-/compare/v0.2.0...v0.3.0

### 📦 Dependencies

- (deps) Update opentalk ([!108](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/108))

## [0.2.0] - 2026-07-16

[0.2.0]: https://git.opentalk.dev/opentalk/backend/services/transcription/-/commits/v0.2.0

### 🚀 New features

- Add dockerfile for local development ()
- Add service probe to query for the state of the service ()
- Add metrics and model endpoint ([!1](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/1))
- Room info in body of start request ([!2](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/2))
- Git-lfs with models ([!3](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/3))
- Add signaling client module ([!4](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/4))
- Intel sycl dockerfile ([!9](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/9))
- Send start & stopped event ([!14](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/14))
- Add simple noise detection + speech detection ([!11](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/11))
- (ci) Add renovate.json ([!19](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/19))
- (release) Add a justfile with create-release target for release automation ([!50](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/50))
- (ci) Include release creation CI job template ([!55](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/55), [#28](https://git.opentalk.dev/opentalk/backend/services/transcription/-/issues/28))
- (roomserver) Implement roomserver integration ([!72](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/72))
- Integrate orcherstrator client ([!78](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/78))
- Add health command ([!85](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/85), [#22](https://git.opentalk.dev/opentalk/backend/services/transcription/-/issues/22))
- Configure logging with default directives ([!93](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/93))

### 🐛 Bug fixes

- Remove accidentially added unwraps from main ([!8](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/8))
- Do not track participant track mapping separatly ([!10](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/10))
- Use tagged version for whisper.cpp in docker build ([!15](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/15))
- (ci) Docker whisper.cpp not building ([!16](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/16))
- Environment variable prefix ([!90](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/90))
- Allow ApiSecrets to be supplied via env variables ([!90](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/90))
- Improve error handling and config env prefix ([!93](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/93))

### 📚 Documentation

- Add README with installation dependencies ([!93](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/93))
- Add admin guide and rustdoc comments ([!93](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/93))
- Add fedora instructions ([!103](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/103))

### 🔨 Refactor

- Update to signaling-types 0.48.0 & some general cleanup ([!6](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/6))
- Make voice activity detection (vad) its own module ([!20](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/20))
- Restructure task management in preparation for the orchestrator ([!75](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/75))
- Create transcription and *-web-api crates ([!76](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/76))
- Restructure audio processing into separate modules ([!84](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/84))
- Decluster main.rs into focused modules ([!93](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/93))

### 📦 Dependencies

- (deps) Update rust crate config to v0.15.19 ([!27](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/27))
- (deps) Update rust crate clap to v4.5.53 ([!26](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/26))
- (deps) Update rust crate anyhow to v1.0.100 ([!24](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/24))
- (deps) Update rust crate owo-colors to v4.2.3 ([!29](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/29))
- (deps) Update rust crate log to v0.4.29 ([!28](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/28))
- (deps) Update rust crate axum to v0.8.7 ([!25](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/25))
- (deps) Update rust crate axum to v0.8.8 ([!37](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/37))
- (deps) Update rust crate clap to v4.5.54 ([!38](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/38))
- (deps) Update rust crate tokio to v1.49.0 ([!32](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/32))
- (deps) Update rust crate service-probe to 0.4 ([!34](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/34))
- (deps) Update rust crate clap to v4.5.56 ([!40](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/40))
- (deps) Update rust crate anstream to v0.6.21 ([!23](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/23))
- (deps) Update whisper-rs digest to a812820 ([!22](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/22))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.93.0 ([!31](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/31))
- (deps) Update alpine docker tag to v3.23 ([!30](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/30))
- (deps) Update rust crate anyhow to v1.0.101 ([!42](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/42))
- (deps) Update rust crate opentalk-version to 0.4 ([!33](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/33))
- (deps) Update rust crate clap to v4.5.57 ([!41](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/41))
- (deps) Update rust crate opentalk-client-signaling to 0.2 ([!39](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/39))
- (deps) Add eupl and openssl to allowed licenses ([!39](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/39))
- (deps) Lock file maintenance ([!35](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/35))
- (deps) Update rust crate clap to v4.5.58 ([!43](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/43))
- (deps) Update rust crate env_logger to v0.11.9 ([!44](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/44))
- (deps) Update rust crate anstream to v1 ([!46](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/46))
- (deps) Update rust crate futures to v0.3.32 ([!47](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/47))
- (deps) Lock file maintenance ([!48](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/48))
- (deps) Update rust crate clap to v4.5.59 ([!49](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/49))
- (deps) Update rust crate clap to v4.5.60 ([!51](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/51))
- (deps) Update rust crate anyhow to v1.0.102 ([!52](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/52))
- (deps) Update rust crate owo-colors to v4.3.0 ([!53](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/53))
- (deps) Update rust crate tokio to v1.50.0 ([!56](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/56))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.0 ([!57](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/57))
- (deps) Update rust crate clap to v4.6.0 ([!59](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/59))
- (deps) Update rust crate config to v0.15.21 ([!60](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/60))
- (deps) Update rust crate config to v0.15.22 ([!61](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/61))
- (deps) Update rust crate env_logger to v0.11.10 ([!63](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/63))
- (deps) Lock file maintenance ([!54](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/54))
- (deps) Lock file maintenance ([!65](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/65))
- (deps) Lock file maintenance ([!68](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/68))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.1 ([!67](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/67))
- (deps) Update rust crate axum to v0.8.9 ([!69](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/69))
- (deps) Update rust crate tokio to v1.52.3 ([!70](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/70))
- (deps) Lock file maintenance ([!73](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/73))
- (deps) Update alpine docker tag to v3.24 ([!94](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/94))
- (deps) Update opentalk crates ([!103](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/103))

### ⚙ Miscellaneous

- Remove unused dependencies ()
- Remove unused Silence struct ([!5](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/5))
- Smaller cargo clippy changes ([!5](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/5))
- Remove uneccessary async keyword ([!5](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/5))
- Add ci ([!7](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/7))
- Add -vulkan suffix to docker image tag (for vulkan) ([!13](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/13))
- Add intel build stage for gitlab ci ([!13](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/13))
- Move docker files to ci folder ([!15](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/15))
- Fix example toml ([!12](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/12))
- (lint) Skip unmaintained rust advisory from livekit ([!45](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/45))
- (ci) Remove signature check for commits ([!58](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/58))
- (ci) Update rust container to 1.95.0 ([!78](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/78))

### Ci

- Use changelog preview template ([!66](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/66))
- (renovate) Add renovate groups ([!103](https://git.opentalk.dev/opentalk/backend/services/transcription/-/merge_requests/103))
- (just) Update changelog script ()
