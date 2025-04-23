# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-04-23

[0.2.0]: https://git.opentalk.dev/opentalk/backend/libs/opentalk-version/-/compare/v0.1.0...v0.2.0

### 🚀 New features

- Print license information ([!7](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/7))
- Add InfoArgs struct that can be flattened into args for clap ([!8](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/8))
- Use a single print function for license and version ([!30](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/30))

### 🐛 Bug fixes

- Use lowercase for 'cargo' in output to match the current behavior of the controller ([!6](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/6))

### 📚 Documentation

- Add crate description ([!4](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/4))

### 📦 Dependencies

- (deps) Lock file maintenance ([!14](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/14), [!15](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/15), [!16](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/16), [!22](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/22), [!26](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/26), [!29](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/29))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.86.0 ([!19](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/19))
- (deps) Update rust crate anyhow to v1.0.98 ([!27](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/27))
- (deps) Update rust crate vergen to v9.0.6 ([!23](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/23))
- (deps) Update rust crate vergen-gix to v1.0.9 ([!24](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/24))

### ⚙ Miscellaneous

- Configure renovate ([!9](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/9))
- Add pre-commit config ([!20](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/20))

### Ci

- Add libs group as renovate reviewers ([!21](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/21))
- Ignore `RUSTSEC-2025-0021` since there is no update yet ([!22](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/22))

## [0.1.0] - 2024-12-17

[0.1.0]: https://git.opentalk.dev/opentalk/backend/libs/opentalk-version/-/compare/de2780ca28529b7806d54bdcb7a71c581c30bd87...v0.1.0

### 🚀 New features

- Collect build information and provide `BuildInfo` struct ([!2](https://git.opentalk.dev/opentalk/backend/libs/opentalk-version/-/merge_requests/2))

### 📚 Documentation

- Add documentation ([!2](https://git.opentalk.dev/opentalk/backend/libs/opentalk-version/-/merge_requests/2))

### ⚙ Miscellaneous

- Project setup ([!1](https://git.opentalk.dev/opentalk/backend/libs/opentalk-version/-/merge_requests/1))
- Rename `ot-version` to `opentalk-version` ([!2](https://git.opentalk.dev/opentalk/backend/libs/opentalk-version/-/merge_requests/2))
