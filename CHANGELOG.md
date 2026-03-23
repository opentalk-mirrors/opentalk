# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [26.1.0-beta.2] - 2026-03-23

### 📦 Dependencies

- (deps) Update OpenTalk Product version v26.1.0-beta.2 ([!141](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/141))
- (deps) Update OpenTalk Controller v0.33.1 ([!141](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/141))
- (deps) Update OpenTalk Obelisk v0.21.2 ([!141](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/141))

### ⚙ Miscellaneous

- (docs) Improve TURN and firewall config ([!139](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/139))

## [26.1.0-beta] - 2026-03-18

### 🚀 New features

- (livekit) Add LIVEKIT_PREFERRED_VIDEO_CODEC ([!137](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/137))

### 🐛 Bug fixes

- (keycloak) Enable backchannel logout ([!135](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/135))
- (minio) Minio healthcheck broken ([!133](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/133))

### 📦 Dependencies

- (deps) Update OpenTalk Product version v26.1.0-beta ([!140](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/140))
- (deps) Update OpenTalk Frontend v2.8.0 ([!140](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/140))
- (deps) Update OpenTalk Controller v0.33.0 ([!140](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/140))
- (deps) Update OpenTalk Obelisk v0.21.0 ([!140](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/140))
- (deps) Update OpenTalk SMTP Mailer v0.17.0 ([!140](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/140))
- (deps) Update OpenTalk Recorder v0.16.1 ([!140](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/140))

### ⚙ Miscellaneous

- (docs) Update CHANGELOG ([!132](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/132))
- (obelisk) Change max video bitrate ([!138](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/138))
- (ci) Enable Rocky Linux 10 test ([!134](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/134))
- (ansible) Check compose.yml before managing a service ([!116](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/116))
- (ansible) Correct path handling for compose includes check ([!116](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/116))
- (docs) Update config reference ([!140](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/140))

## [25.4.6] - 2026-03-11

### 🐛 Bug fixes

- (spacedeck) Adjust spacedeck healthcheck ([!131](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/131))

### 📦 Dependencies

- (deps) Update OpenTalk Product version v25.4.6 ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (deps) Update OpenTalk Frontend v2.7.5 ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (deps) Update OpenTalk Controller v0.32.7 ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (deps) Update traefik v3.6.10 ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (deps) Update postgres 17.9 ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (deps) Update keycloak 26.5.5 ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (deps) Update rabbitmq 4.2.4 ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (deps) Update livekit v1.9.12 ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (deps) Update minio RELEASE.2025-10-15T17-29-55Z ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))

### ⚙ Miscellaneous

- (docs) Update CHANGELOG ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (container) Heinlein registry by default ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))
- (docs) Update CHANGELOG ([!128](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/128))

## [25.4.5] - 2026-03-05

[25.4.5]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.4...v25.4.5

### 🐛 Bug fixes

- (keycloak) Force demo user password update ([!126](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/126))

### 📦 Dependencies

- (deps) Update dependency ansible-core to v2.20.3 ([!122](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/122))
- (deps) Update dependency community.general to v12.4.0 ([!121](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/121))
- (deps) Update OpenTalk Product version v25.4.5 ([!127](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/127))
- (deps) Update OpenTalk Controller v0.32.6 ([!127](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/127))
- (deps) Update OpenTalk Frontend v2.7.4 ([!127](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/127))
- (deps) Update Etherpad v2.0.3 ([!127](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/127))

### ⚙ Miscellaneous

- (docs) Update CHANGELOG ([!125](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/125))

## [25.4.4] - 2026-03-04

[25.4.4]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.3...v25.4.4

### 🚀 New features

- (keycloak) Make keycloak realm session timeouts configurable ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (outlook-add-in) Make oidc scopes configurable ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (packaging) Add keycloak-theme-generator binary to the opentalk-compose package ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (keycloak) Make keycloak theme configurable ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))

### 🐛 Bug fixes

- (docs) Correct config variable name for TURN TLS ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))

### 📦 Dependencies

- (deps) Update dependency ansible-core to v2.20.3 ([!122](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/122))
- (deps) Update dependency community.general to v12.4.0 ([!121](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/121))
- (deps) Update dependency community.docker to v5.0.6 ([!107](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/107))
- (deps) Update dependency community.crypto to v3.1.1 ([!118](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/118))
- (deps) Update keycloak to version 26.5.3 ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (deps) Update keycloak-theme-generator-tool to v0.1.2 ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (deps) Update web frontend to version v2.7.3 ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (deps) Update controller to version v0.32.5 ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (deps) Update obelisk to version v0.20.12 ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (deps) Update outlook-add-in to version v0.20.12 ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))

### ⚙ Miscellaneous

- (keycloak) Support session timeout variables in realm config template ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (keycloak) Outlook addin client session timeouts inherited from realm ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (keyclaok) Reset client session timeouts before realm update ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (keycloak) Restructure realm tasks to reset clients before timeouts ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (docs) Update image source defaults ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (container) Switch to registry.heinlein.group as default image source for opentalk components ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (ci) Experimental deployment is optional ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (docs) Change livekit server default port range to 50000-65535/udp ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (keyclaok) Make access token lifespan configurable ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (docs) Add access token lifespan defaults ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (packaging) Use Job artifacts to fetch keycloak-theme-generator ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (docs) Add keycloak-theme-generator-tool docs ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))
- (docs) Set correct settings path in installation docs ([!124](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/124))

## [25.4.3] - 2026-02-11

[25.4.3]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.2...v25.4.3

### 📦 Dependencies

- (deps) Livekit-server v1.9.11 ([!104](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/104))

### ⚙ Miscellaneous

- Release v25.4.3 ([!104](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/104))

## [25.4.2] - 2026-02-10

[25.4.2]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.1-3...v25.4.2

### 🚀 New features

- (ci) Add deployment stage to GitLab CI pipeline ([!102](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/102))
- (ci) Use deployment target hosts from variable ([!102](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/102))
- (deps) Opentalk controller v0.32.3 ([!103](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/103))

### 📦 Dependencies

- (deps) Update dependency ansible-core to v2.20.2 ([!101](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/101))
- (deps) Release OpenTalk v25.4.2 ([!103](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/103))

### ⚙ Miscellaneous

- (ansible) Wait for keycloak in demo user task ([!102](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/102))

## [25.4.1] - 2026-01-23

[25.4.1]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0-4...v25.4.1

### 🚀 New features

- (docker compose) Ulimit settings via docker compose ([!92](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/92))

### 📚 Documentation

- Add CHANGELOG.md file ([!91](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/91))
- Add ulimits defaults to config reference ([!92](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/92))

### 📦 Dependencies

- (deps) Update dependency community.crypto to v3.1.0 ([!88](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/88))
- (deps) Update dependency community.general to v12.2.0 ([!89](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/89))
- (deps) Update opentalk product version v25.4.1 ([!96](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/96))

### ⚙ Miscellaneous

- Add changelog creation ([!91](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/91))
- (fix) Missing ulimit settings for database services ([!97](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/97))
- (fix) Missing ulimit settings for rabbitmq ([!99](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/99))

## [25.4.0-4] - 2026-01-05

[25.4.0-4]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0-3...v25.4.0-4

### 📦 Dependencies

- (deps) Update dependency community.crypto to v2.26.6 ([!76](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/76))
- (deps) Update dependency community.docker to v4.8.5 ([!77](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/77))
- (deps) Update dependency community.docker to v5 ([!80](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/80))
- (deps) Update dependency community.crypto to v3 ([!79](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/79))
- (deps) Update dependency ansible-core to v2.20.1 ([!82](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/82))
- (deps) Update dependency bcrypt to v5 ([!84](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/84))
- (deps) Update dependency astral-sh/python-build-standalone to v20251217 ([!85](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/85))
- (deps) Update dependency community.docker to v5.0.5 ([!90](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/90))

## [25.4.0-3] - 2025-12-19

[25.4.0-3]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0-2...v25.4.0-3

### ⚙ Miscellaneous

- Update product version check ()

## [25.4.0-2] - 2025-12-17

[25.4.0-2]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0...v25.4.0-2

### 🚀 New features

- (ci) Add renovate.json ([!72](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/72))

### 📦 Dependencies

- (deps) Update dependency community.general to v12 ([!81](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/81))

### ⚙ Miscellaneous

- Exclude renovate.json from packaging ()
- Update Quickstart guide ([!74](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/74))
- Adjust requirements.yml ()
- Extend renovate configuration ([!75](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/75))

## [25.4.0-4] - 2026-01-05

[25.4.0-4]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0-3...v25.4.0-4

### 📦 Dependencies

- (deps) Update dependency community.crypto to v2.26.6 ([!76](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/76))
- (deps) Update dependency community.docker to v4.8.5 ([!77](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/77))
- (deps) Update dependency community.docker to v5 ([!80](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/80))
- (deps) Update dependency community.crypto to v3 ([!79](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/79))
- (deps) Update dependency ansible-core to v2.20.1 ([!82](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/82))
- (deps) Update dependency bcrypt to v5 ([!84](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/84))
- (deps) Update dependency astral-sh/python-build-standalone to v20251217 ([!85](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/85))
- (deps) Update dependency community.docker to v5.0.5 ([!90](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/90))

## [25.4.0-3] - 2025-12-19

[25.4.0-3]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0-2...v25.4.0-3

### ⚙ Miscellaneous

- Update product version check ()

## [25.4.0-2] - 2025-12-17

[25.4.0-2]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0...v25.4.0-2

### 🚀 New features

- (ci) Add renovate.json ([!72](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/72))

### 📦 Dependencies

- (deps) Update dependency community.general to v12 ([!81](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/81))

### ⚙ Miscellaneous

- Exclude renovate.json from packaging ()
- Update Quickstart guide ([!74](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/74))
- Adjust requirements.yml ()
- Extend renovate configuration ([!75](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/75))

## [25.4.1] - 2026-01-22

[25.4.1]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0-4...v25.4.1

### 🚀 New features

- (docker compose) Ulimit settings via docker compose ([!92](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/92))

### 📚 Documentation

- Add CHANGELOG.md file ([!91](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/91))
- Add ulimits defaults to config reference ([!92](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/92))

### 📦 Dependencies

- (deps) Update dependency community.crypto to v3.1.0 ([!88](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/88))
- (deps) Update dependency community.general to v12.2.0 ([!89](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/89))
- (deps) Update opentalk product version v25.4.1 ([!96](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/96))

### ⚙ Miscellaneous

- Add changelog creation ([!91](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/91))

## [Unreleased]

### 📦 Dependencies

- (deps) Update dependency community.crypto to v3.1.0 ([!88](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/88))

### ⚙ Miscellaneous

- Add changelog creation ([!91](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/91))

## [25.4.0-4] - 2026-01-05

[25.4.0-4]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0-3...v25.4.0-4

### 📦 Dependencies

- (deps) Update dependency community.crypto to v2.26.6 ([!76](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/76))
- (deps) Update dependency community.docker to v4.8.5 ([!77](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/77))
- (deps) Update dependency community.docker to v5 ([!80](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/80))
- (deps) Update dependency community.crypto to v3 ([!79](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/79))
- (deps) Update dependency ansible-core to v2.20.1 ([!82](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/82))
- (deps) Update dependency bcrypt to v5 ([!84](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/84))
- (deps) Update dependency astral-sh/python-build-standalone to v20251217 ([!85](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/85))
- (deps) Update dependency community.docker to v5.0.5 ([!90](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/90))

## [25.4.0-3] - 2025-12-19

[25.4.0-3]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0-2...v25.4.0-3

### ⚙ Miscellaneous

- Update product version check ()

## [25.4.0-2] - 2025-12-17

[25.4.0-2]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.0...v25.4.0-2

### 🚀 New features

- (ci) Add renovate.json ([!72](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/72))

### 📦 Dependencies

- (deps) Update dependency community.general to v12 ([!81](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/81))

### ⚙ Miscellaneous

- Exclude renovate.json from packaging ()
- Update Quickstart guide ([!74](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/74))
- Adjust requirements.yml ()
- Extend renovate configuration ([!75](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/75))

## [25.4.0] - 2025-12-15

[25.4.0]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/commits/v25.4.0

### 🚀 New features

- Start controller external example ([!18](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/18))
- Allow user write to config files ([!18](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/18))
- Enable ci pipeline ([!59](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/59))
- Support controller cors config ([!62](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/62))
- Wait for keycloak when it is not ready ([!65](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/65))
- Add integration test ([!67](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/67))
- Add user profile config ([!69](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/69))
- Mkdocs integration ([!71](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/71))

### 🐛 Bug fixes

- Markdown link ([!59](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/59))
- Mailer ansible does not properly turn off ([!60](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/60))
- Template rendering for screen_share_requires_permission ([!61](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/61))
- Zoneinfo field missing from Keycloak user attributes, closes #83 ([!69](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/69))

### ⚙ Miscellaneous

- Remove duplicate OBELISK_IMAGE_SRC entry ([!23](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/23))
- Keep linter happy ([!59](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/59))
- Update linter & pre-commit config ([!64](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/64))
- Add condition for managing smtp mailer config ([!66](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/66))
- Update tag versions ([!65](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/65))
- Update linter & pre-commit config ([!65](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/65))
- Error handling for commitlint ([!65](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/65))
- Individual secret length ([!66](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/66))
- Run depending on service availability ([!66](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/66))
- Python passlib removed; installed in runtime ([!66](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/66))
- Reset test values ([!66](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/66))
- Update pipeline config ([!67](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/67))
- Test gitlab pipeline rules ()
- Add nexus upload ()
- Update file path ()
- Update curl options ()
- Update image versions related to v25.4.0-beta.3 ([!69](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/69))
- Exclude linter configs from packaging ([!68](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/68))
- Add repo name variables ([!69](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/69))
- Remove TOC and fix typo ([!71](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/71))
- Use content tabs with install instructions ([!71](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/71))
- Update product version ([!73](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/73))

