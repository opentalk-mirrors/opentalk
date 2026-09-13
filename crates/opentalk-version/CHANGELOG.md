# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-01-19

[0.4.0]: https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/compare/v0.3.0...v0.4.0

### 📦 Dependencies

- (deps) Lock file maintenance ([!109](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/109), [!110](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/110), [!112](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/112), [!118](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/118))
- (deps) Update `vergen-git2` to v9.1.0 ([!118](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/118))
- (deps) Update alpine docker tag to v3.23 ([!106](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/106))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.92.0 ([!108](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/108))
- (deps) Update pre-commit hook adrienverge/yamllint to v1.38.0 ([!114](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/114))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.24.0 ([!115](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/115))
- (deps) Update pre-commit hook andrejorsula/pre-commit-cargo to v0.5.0 ([!111](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/111))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.0 ([!113](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/113))
- (deps) Update pre-commit hook markdownlint/markdownlint to v0.15.0 ([!104](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/104))

## [0.3.0] - 2025-10-27

[0.3.0]: https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/compare/v0.2.0...v0.3.0

### 🐛 Bug fixes

- (clippy) Fix push format str clippy rule ([!37](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/37))
- Compiler error ([!97](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/97))
- (just) Only set version of `opentalk-version` crate ([!98](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/98))
- (just) Use correct version number in `commit-release` ([!98](https://git.opentalk.dev/opentalk/backend/libs/opentalk-version/-/merge_requests/98))

### 📚 Documentation

- Update example usage ([!97](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/97))

### 🔨 Refactor

- Simplify is_contained_in_git function ([!33](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/33))
- Use `vergen-git2` instead of `vergen-gix` ([!94](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/94))

### 📦 Dependencies

- (deps) Lock file maintenance ([!32](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/32), [!34](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/34), [!36](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/36), [!39](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/39), [!40](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/40), [!43](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/43), [!44](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/44), [!46](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/46), [!47](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/47), [!49](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/49), [!51](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/51), [!56](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/56), [!57](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/57), [!59](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/59), [!63](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/63), [!68](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/68), [!70](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/70), [!72](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/72), [!76](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/76), [!77](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/77), [!80](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/80), [!83](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/83), [!85](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/85), [!88](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/88), [!90](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/90), [!92](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/92))
- (deps) Update alpine docker tag to v3.22 ([!42](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/42))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.90.0 ([!81](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/81))
- (deps) Update pre-commit hook adrienverge/yamllint to v1.37.1 ([!53](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/53))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.23.0 ([!84](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/84))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.5 ([!82](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/82))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6.2.0 ([!93](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/93))
- (deps) Update pre-commit hook pre-commit/pre-commit-hooks to v6 ([!62](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/62))
- (deps) Update rust crate anyhow to v1.0.100 ([!95](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/95))
- (deps) Update rust crate clap to v4.5.50 ([!96](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/96))

### ⚙ Miscellaneous

- (ci) Update default image to Debian Trixie ([!69](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/69), [#2](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/issues/2))

### Test

- Add canary crate to ensure the code compiles ([!97](https://git.opentalk.dev/opentalk/backend/libs/ot-version/-/merge_requests/97))

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
