# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [26.3.0] - 2026-10-28

[26.3.0]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.3.0-beta.2...v26.3.0

### 🐛 Bug fixes

- (ci) Deploy testing hosts from tag pipeline packages ([!257](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/257))

### 📦 Dependencies

- (deps) Update OpenTalk product version v26.3.0 ([!260](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/260))

## [26.3.0-beta.2] - 2026-10-26

[26.3.0-beta.2]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.3.0-beta.1...v26.3.0-beta.2

### 🐛 Bug fixes

- (livekit) Exclude link-local ipv6 from ice candidates ([!251](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/251))

### 📦 Dependencies

- (deps) Update OpenTalk web frontend version v2.10.2 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update OpenTalk controller version v0.35.3 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update OpenTalk obelisk version v0.23.2 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update OpenTalk recorder version v0.18.2 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update OpenTalk roomserver version v0.10.3 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update OpenTalk smtp-mailer version v0.19.1 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update OpenTalk orchestrator version v0.12.1 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update OpenTalk outlook add-In version v0.6.0 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update keycloak version v26.8.1-1 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update rabbitmq version v4.3.7 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update NDT server version v0.26.1 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update garage version v2.5.0 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))
- (deps) Update OpenTalk product version v26.3.0-beta.2 ([!254](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/254))

## [26.3.0-beta.1] - 2026-10-22

[26.3.0-beta.1]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.2.1...v26.3.0-beta.1

### 🐛 Bug fixes

- (obelisk) Reach the controller directly for the internal call-in api ([!242](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/242))
- (compose) Add orchestrator storage kind ([!247](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/247))
- (compose) Create the compose-python venv with the pinned python minor release ([!248](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/248))
- (e2e) Move the traefik ping entry point off the ingress port ([!248](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/248))
- (ndt) Restart the container like the other services ([!244](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/244))
- (controller) Deny /controller/metrics in traefik ([!245](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/245))

### 📦 Dependencies

- (deps) Update OpenTalk web frontend version v2.10.1 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))
- (deps) Update OpenTalk controller version v0.35.2 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))
- (deps) Update OpenTalk obelisk version v0.23.1 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))
- (deps) Update OpenTalk recorder version v0.18.1 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))
- (deps) Update OpenTalk roomserver version v0.10.2 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))
- (deps) Update livekit-server version v1.13.10 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))
- (deps) Update keycloak version v26.8.1 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))
- (deps) Update traefik version v3.7.16 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))
- (deps) Update OpenTalk product version v26.3.0-beta.1 ([!249](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/249))

## [26.3.0-beta.0] - 2026-10-05

[26.3.0-beta.0]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/commits/v26.3.0-beta.0

### 🚀 New features

- (dev) Add local compose development workflow ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Add ansible playbook syntax check to lint stage ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (otctl) Add restore with safe DB, config and volume restore tasks ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Add shell test lint job and integration tests to all int pipelines ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Add manual E2E Compose test job ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (coverage) Add python coverage measurement for config-reference generator ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (packaging) Read pins from .versions.yml ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Add rocky10 with-optional-services integration slot ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Expose separate coverage metrics ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Add local pkgbuild image ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Derive product version from git tag ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Trigger downstream docs build ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (controller) Append tokens to room aliases by default ([!225](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/225))
- (controller) Configure room alias suffixes ([!227](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/227))
- (migrate) Detect pending migrations and guide the user to them ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (postgres) Upgrade to postgresql 18 with a guided migration ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (s3-migrate) Start the destination and deploy from otctl ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))

### 🐛 Bug fixes

- (backup) Quote database backup command arguments ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (keycloak) Keep openid out of default client scopes ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (keycloak) Make client scopes idempotent ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (smtp-mailer) Remove process limit ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (keycloak) Accept 400 and make E2E test port-configurable ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (coverage) Exclude here-doc bodies from shell coverage denominator ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (otctl) Keep rows through inspect fallback loop; test status fallbacks and dpkg path ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (coverage) Simplify control-flow exclusion, drop function defs and bare case patterns ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (packaging) Grep/find exit-code bugs in 13/31/80 ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (packaging) Make --auto-revision work ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (docs) Fix broken configuration-reference anchors ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (pkg) Include staged payload in fpm builds ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (pkg) Validate embedded runtime in deb payload ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (pkg) Normalize embedded venv python links ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (pkg) Rebase embedded venv script shebangs ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (compose) Avoid keycloak ingress port with traefik ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Use reachable integration test hostname ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Route integration OIDC discovery through a resolvable test domain ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Retry mkdocs asset fetch and ansible-galaxy collection download ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (obelisk) Remove deprecated Keycloak OIDC client and secret ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Drop removed obelisk OIDC client from keycloak idempotency test ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Sync config reference after rebase ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (orchestrator) Render public url in config ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Support experimental uploads without tag ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Support installed yq version ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (e2e) Stabilize compose setup and keycloak ingress ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (dev) Route keycloak through traefik instead of unreachable direct port ([!223](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/223))
- Add missing roomserver path to traefik and fix livekit turn config closes #157 ([!213](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/213))
- (docs) Align config reference generator with merged defaults ([!224](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/224))
- (ci) Lint merge request commits individually ([!224](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/224))
- (ci) Repair post-merge role-name check and stale test expectation ([!226](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/226))
- (otctl) Stop migrate --no-deploy from passing an empty argument ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (s3-migrate) Stop the controller while objects are copied ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (postgres) Let images without root read the password file ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (packaging) Extract rpm payloads without a pipe in verify-packages ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))

### 📚 Documentation

- Update local development guidance ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- Add backup restore section with test procedure ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- Escape macro like strings `{{ .. }}` ([!214](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/214))
- (changelog) Update v26.2.1 release notes ([!224](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/224))
- Add upgrades and migrations chapter as chapter 2 ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))

### 🔨 Refactor

- (packaging) Rework embedded runtime packaging ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ansible) Normalize role names ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Use descriptive matrix value for optional-services slot ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Use debian13 instead of EOL debian12 for optional-services slot ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Build and use commit-pinned local pkgbuild image ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Cache local pkgbuild image and avoid duplicate pipelines ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Reuse pkgbuild image between pipeline runs ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Extract lint jobs ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Run docs and package builds through just ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Extract package jobs ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Extract deployment jobs ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Extract integration jobs ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Extract shared package template and fix CI contract test ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Extract shared base templates for upload and integration jobs ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Extract shared scripts and finalize optimizations ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Extract integration test runner ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Reuse release package and tag checks ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Move integration settings to file ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ci) Separate manual deployment stage ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))

### 📦 Dependencies

- (deps) Update dependency community.crypto to v3.4.0 ([!209](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/209))
- (deps) Update dependency community.docker to v5.3.0 ([!210](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/210))
- (deps) Update dependency community.general to v13.4.0 ([!211](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/211))
- (deps) Update dependency ansible-core to v2.21.4 ([!212](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/212))
- (deps) Update OpenTalk product version v26.3.0-beta.0 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update OpenTalk web frontend version v2.10.0 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update OpenTalk controller version v0.35.1 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update OpenTalk obelisk version v0.23.0 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update OpenTalk recorder version v0.18.0 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update OpenTalk roomserver version v0.10.1 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update OpenTalk smtp-mailer version v0.19.0 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update OpenTalk orchestrator version v0.12.0 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update OpenTalk outlook add-In version v0.5.0 ([!228](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/228))
- (deps) Update livekit-server version v1.13.7 ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (deps) Update keycloak version v26.8.0 ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (deps) Update rabbitmq version v4.3.6 ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (deps) Update traefik version v3.7.13 ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (deps) Update garage version v2.4.1 ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))
- (deps) Update postgres version v18.6.0 ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))

### ⚙ Miscellaneous

- (versions) Bump uv to 0.12.3 ([!201](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/201))
- (ansible) Print task results as yaml ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))

### Test

- (ci) Run the postgres migration and the upgrade from stable in ci ([!237](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/237))

## [26.2.1] - 2026-09-22

[26.2.1]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.2.1-beta.0...v26.2.1

### 🐛 Bug fixes

- (ci) Only lint HEAD commit on push pipelines in commitlint job ([!208](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/208))
- (keycloak) Handle 400 response in permanent admin check ([!208](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/208))
- Add missing roomserver path to traefik and fix livekit turn config closes #157 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (docs) Align config reference generator with merged defaults ([!222](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/222))


### 📚 Documentation

- (quickstart) Fix gpgkey URL ([!208](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/208))
- Escape macro like strings `{{ .. }}` ([!216](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/216))

### 📦 Dependencies

- (deps) Update dependency community.crypto to v3.3.0 ([!208](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/208))
- (deps) Update dependency community.docker to v5.2.2 ([!208](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/208))
- (deps) Update dependency ansible-core to v2.21.3 ([!208](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/208))
- (deps) Update dependency community.general to v13.3.0 ([!208](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/208))
- (deps) Update Opentalk product version v26.2.1-beta.0 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk controller version v0.34.5 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk recorder version v0.17.5 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk roomserver version v0.1.4 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk orchestrator version v0.8.3 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk product version to v26.2.1 ([!221](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/221))
- (deps) Update changelog for v26.2.1 ([!221](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/221))

### ⚙ Miscellaneous

- Update changelog for v26.2.1-beta.0 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))


## [26.2.1-beta.0] - 2026-09-16

[26.2.1-beta.0]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.2.0...v26.2.1-beta.0

### 🐛 Bug fixes

- (quickstart) Fix gpgkey URL ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (keycloak) Handle 400 response in permanent admin check ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (ci) Only lint HEAD commit on push pipelines in commitlint job ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))

### 📦 Dependencies

- (deps) Update OpenTalk product version v26.2.1-beta.0 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk controller version v0.34.5 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk recorder version v0.17.5 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk roomserver version v0.1.4 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update OpenTalk orchestrator version v0.8.3 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update dependency community.docker to v5.2.2 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update dependency ansible-core to v2.21.3 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update dependency community.general to v13.3.0 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))
- (deps) Update dependency community.crypto to v3.3.0 ([!219](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/219))

## [26.2.0] - 2026-08-31

[26.2.0]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.1.2...v26.2.0

### 🚀 New features

- (roomserver) Add roomserver service ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Add opentalk orchestrator service ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Derive service state from controller_roomserver_kind ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (web-frontend) Align environment variables with frontend documentation ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))

### 🐛 Bug fixes

- (docs) Fix bad escaping of | in configuration options reference ([!190](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190))
- (docs) Add garage and s3_migrate sections, fix duplicate minio_s3_api_port entry ()
- (web-frontend) Make video_backgrounds default-friendly ([!167](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/167))
- (roomserver) Update default port ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (controller) Keep meeting-notes base_url in sync with etherpad ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (controller) Enable recording defaults and service URL ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (livekit) Use ansible_facts for bind address detection ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (keycloak) Enforce audience check for controller introspection ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (ci) Skip disabled roomserver compose management ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (web-frontend) Add fallbacks for opentalk_web_frontend_backgrounds_dir ([!202](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/202))
- (obelisk) Migrate controller config to v0.22.2 url/api_key format ([!202](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/202))

### 📚 Documentation

- Improve install and upgrade guidance ()
- (Changelog) Update Changelog ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))

### 🔨 Refactor

- (traefik) Rename csp_allowed_extra_*_src vars to traefik_ prefix ([!166](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/166))
- (ansible) Align opentalk_orchestrator role with current patterns ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))

### 📦 Dependencies

- (deps) Update OpenTalk outlook add-in version v0.3.6-2 ([!190](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190))
- (deps) Update NDT server version v0.25.3 ([!190](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190))
- (deps) Update dependency community.docker to v5.2.1 ([!177](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/177))
- (deps) Update dependency community.crypto to v3.2.1 ([!172](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/172))
- (deps) Update dependency ansible.posix to v2.2.2 ([!191](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/191))
- (deps) Update dependency ansible-core to v2.21.2 ([!187](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/187))
- (deps) Update dependency community.general to v13.2.0 ([!185](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/185))
- (deps) Update OpenTalk recorder version v0.17.1 ()
- (deps) Add OpenTalk orchestrator version v0.8.1 ()
- (deps) Update traefik version v3.7.9 ()
- (deps) Update garage version v2.3.0 ([!190](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190))
- (deps) Update livekit-server version v1.13.5 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update keycloak version v26.7.2 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))
- (deps) Update postgres version v17.11.0 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update rabbitmq version v4.3.4 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update mailcrab version v1.9.0 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update OpenTalk web frontend version v2.9.2 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))
- (deps) Update OpenTalk controller version v0.34.4 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))
- (deps) Update OpenTalk obelisk version v0.22.2 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update OpenTalk roomserver version v0.1.3 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))
- (deps) Update OpenTalk smtp-mailer version v0.18.0 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update OpenTalk product version v26.2.0 ([!206](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/206))

### ⚙ Miscellaneous

- (docs) Update CHANGELOG ([!190](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190))
- (docs) Update CHANGELOG.md ([!181](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/181))
- (livekit) Support config option bind_addresses ()
- (docs) Update config referernce ()
- (controller) Update roomserver config template ()
- (ansible) Rename roomserver role directory ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Include roomserver role in opentalk playbook ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Remove roomserver-db service from container state mgmnt ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (packaging) Add roomserver defaults to settings.yml ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Add roomserver to versions file ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ci) Enable roomserver in integration test ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ci) Auth docker client to registry ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Add roomserver default config ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ci) Override web frontend image source in integration test ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (frontend) Change image source ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (controller) Update livekit api keys for roomserver ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Add opentalk orchestrator to default playbook ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Change orchestrator task file name ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Change orchestrator secret defaults ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ci) Disable orchestrator in integration test ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Add orchestrator default config ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Adjust compose config ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (spacedeck) Adjust config to support alpine base image ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Absent by default ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Present by default ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Switch to sh shell ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (web-frontend) Use production image source as default ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (controller) Add new option controller_roomserver_kind ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Add conference section and update config options ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (controller) Add missing settings to controller template ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (controller) Align controller configuration with TOML example ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (ci) Roomsercer container disabled by default ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (obelisk) Remove deprecated Keycloak OIDC client and secret ([!202](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/202))
- Update changelog for v26.2.0-beta.4 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))

## [26.2.0-beta.4] - 2026-08-27

[26.2.0-beta.4]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.1.2...v26.2.0-beta.4

### 🐛 Bug fixes

- (ci) Skip disabled roomserver compose management ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (web-frontend) Add fallbacks for opentalk_web_frontend_backgrounds_dir ([!202](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/202))
- (obelisk) Migrate controller config to v0.22.2 url/api_key format ([!202](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/202))

### 📚 Documentation

- (Changelog) Update Changelog ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))

### 📦 Dependencies

- (deps) Update OpenTalk product version v26.2.0-beta.4 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))
- (deps) Update OpenTalk web frontend version v2.9.2 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))
- (deps) Update OpenTalk controller version v0.34.4 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))
- (deps) Update OpenTalk roomserver version v0.1.3 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))
- (deps) Update keycloak version v26.7.2 ([!205](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/205))

### ⚙ Miscellaneous

- (obelisk) Remove deprecated Keycloak OIDC client and secret ([!202](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/202))

## [26.1.2] - 2026-08-26

[26.1.2]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.1.1...v26.1.2

### 📦 Dependencies

- (deps) Update OpenTalk product version v26.1.2 ([!203](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/203))
- (deps) Update keycloak version v26.7.2 ([!203](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/203))
- (deps) Update postgres version v17.11.0 ([!203](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/203))
- (deps) Update traefik version v3.7.11 ([!203](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/203))
- (deps) Update rabbitmq version v4.2.9 ([!203](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/203))

## [26.2.0-beta.2] - 2026-08-20

[26.2.0-beta.2]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.1.1...v26.2.0-beta.2

### 📦 Dependencies

- (deps) Update OpenTalk product version v26.2.0-beta.2 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update OpenTalk web frontend version v2.9.1 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update OpenTalk controller version v0.34.2 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update OpenTalk obelisk version v0.22.2 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update OpenTalk roomserver version v0.1.1 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update OpenTalk smtp-mailer version v0.18.0 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update livekit-server version v1.13.5 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update keycloak version v26.7.1 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update postgres version v17.11.0 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update rabbitmq version v4.3.4 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))
- (deps) Update mailcrab version v1.9.0 ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))

### ⚙ Miscellaneous

- (ci) Roomsercer container disabled by default ([!200](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/200))

## [26.1.1] - 2026-08-11

[26.1.1]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v26.1.1-beta.2...v26.1.1

### 🐛 Bug fixes

- (docs) Fix bad escaping of | in configuration options reference ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (keycloak) Enforce audience check for controller introspection ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (docs) Add garage and s3_migrate sections, fix duplicate minio_s3_api_port entry ()
- (web-frontend) Make video_backgrounds default-friendly ([!167](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/167))

### 📚 Documentation

- (quickstart) Switch package repository examples to stable ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (changelog) Prepare v26.1.1 release notes ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (changelog) Remove duplicate dependency entries ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))

### 🔨 Refactor

- (traefik) Rename csp_allowed_extra_*_src vars to traefik_ prefix ([!166](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/166))

### 📦 Dependencies

- (deps) Update keycloak version 26.6.3 ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (deps) Update postgresql version 17.10 ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (deps) Update traefik version v3.7.7 ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (deps) Update garage version v2.3.0 ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (deps) Update mailcrab version v1.7.0 ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (deps) Update NDT server version v0.25.3 ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (deps) Update opentalk product version to v26.1.1 ([!199](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/199))
- (deps) Update OpenTalk Controller version 0.33.4 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update OpenTalk Outlook Add-In version v0.3.6-1 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update RabbitMQ to version v4.2.7 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update dependency community.docker to v5.2.1 ([!177](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/177))
- (deps) Update dependency ansible-core to v2.21.0 ([!176](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/176))
- (deps) Update dependency community.general to v13 ([!175](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/175))
- (deps) Update dependency ansible.posix to v2.2.0 ([!174](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/174))
- (deps) Update dependency community.crypto to v3.2.1 ([!172](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/172))
- (deps) Update dependency ansible.posix to v2.2.1 ([!188](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/188))
- (deps) Update dependency ansible.posix to v2.2.2 ([!191](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/191))
- (deps) Update dependency ansible-core to v2.21.2 ([!187](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/187))
- (deps) Update dependency community.general to v13.2.0 ([!185](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/185))
- (deps) Update dependency community.crypto to v3.3.0 ([!184](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/184))
- (deps) Update dependency community.docker to v5.2.2 ([!195](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/195))
- (deps) Update dependency ansible-core to v2.21.3 ()
- (deps) Update dependency community.general to v13.3.0 ()

### ⚙ Miscellaneous

- (docs) Update CHANGELOG.md ([!181](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/181))
- (livekit) Support config option bind_addresses ()
- (docs) Update config referernce ()
- (controller) Update roomserver config template ()

## [26.2.0-beta.1] - 2026-08-07

[26.2.0-beta.1]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.6...v26.2.0-beta.1

### 🚀 New features

- (roomserver) Add roomserver service ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Add opentalk orchestrator service ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Derive service state from controller_roomserver_kind ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Integrate roomserver orchestrator ()
- (web-frontend) Align environment variables with frontend documentation ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))

### 🐛 Bug fixes

- (docs) Add garage and s3_migrate sections, fix duplicate minio_s3_api_port entry ()
- (web-frontend) Make video_backgrounds default-friendly ([!167](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/167))
- (roomserver) Update default port ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (controller) Keep meeting-notes base_url in sync with etherpad ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (controller) Enable recording defaults and service URL ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (livekit) Use ansible_facts for bind address detection ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (keycloak) Enforce audience check for controller introspection ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))

### 📚 Documentation

- Improve install and upgrade guidance ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))

### 🔨 Refactor

- (traefik) Rename csp_allowed_extra_*_src vars to traefik_ prefix ([!166](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/166))
- (ansible) Align opentalk_orchestrator role with current patterns ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))

### 📦 Dependencies

- (deps) Update OpenTalk Product version v26.1.0-beta.6 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update OpenTalk Controller version 0.33.4 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update OpenTalk Outlook Add-In version v0.3.6-1 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update Keycloak to version v26.5.7 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update RabbitMQ to version v4.2.7 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update dependency community.docker to v5.2.1 ([!177](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/177))
- (deps) Update dependency ansible-core to v2.21.0 ([!176](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/176))
- (deps) Update dependency community.general to v13 ([!175](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/175))
- (deps) Update dependency ansible.posix to v2.2.0 ([!174](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/174))
- (deps) Update dependency community.crypto to v3.2.1 ([!172](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/172))
- (deps) Update dependency ansible.posix to v2.2.1 ([!188](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/188))
- (deps) Update dependency ansible.posix to v2.2.2 ([!191](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/191))
- (deps) Update dependency ansible-core to v2.21.2 ([!187](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/187))
- (deps) Update dependency community.general to v13.2.0 ([!185](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/185))
- (deps) Update OpenTalk product version 26.2.0-beta.1 ()
- (deps) Update OpenTalk web frontend version v2.9.0 ()
- (deps) Update OpenTalk controller version v0.34.1 ()
- (deps) Update OpenTalk obelisk version v0.22.1 ()
- (deps) Update OpenTalk recorder version v0.17.1 ()
- (deps) Add OpenTalk roomserver version v0.1.0 ()
- (deps) Add OpenTalk orchestrator version v0.8.1 ()
- (deps) Update livekit version v1.13.1 ()
- (deps) Update rabbitmq version v4.3.1-management-alpine ()
- (deps) Update traefik version v3.7.9 ()

### ⚙ Miscellaneous

- (docs) Update CHANGELOG.md ([!181](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/181))
- (livekit) Support config option bind_addresses ()
- (docs) Update config referernce ()
- (controller) Update roomserver config template ()
- (ansible) Rename roomserver role directory ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Include roomserver role in opentalk playbook ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Remove roomserver-db service from container state mgmnt ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (packaging) Add roomserver defaults to settings.yml ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Add roomserver to versions file ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ci) Enable roomserver in integration test ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ci) Auth docker client to registry ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Add roomserver default config ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ci) Override web frontend image source in integration test ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (frontend) Change image source ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (controller) Update livekit api keys for roomserver ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Add opentalk orchestrator to default playbook ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Change orchestrator task file name ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ansible) Change orchestrator secret defaults ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (ci) Disable orchestrator in integration test ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Add orchestrator default config ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Adjust compose config ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (spacedeck) Adjust config to support alpine base image ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Absent by default ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (orchestrator) Present by default ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Switch to sh shell ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (web-frontend) Use production image source as default ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (controller) Add new option controller_roomserver_kind ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (roomserver) Add conference section and update config options ([!123](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/123))
- (controller) Add missing settings to controller template ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))
- (controller) Align controller configuration with TOML example ([!193](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/193))


## [26.1.1-beta.1] - 2026-07-09

[26.1.1-beta.1]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.6...v26.1.1-beta.1

### 🐛 Bug fixes

- (docs) Fix bad escaping of | in configuration options reference (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)

### 📦 Dependencies

- (deps) Update OpenTalk product version v26.1.1-beta.1 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update OpenTalk web frontend version v2.8.5 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update OpenTalk obelisk version v0.21.3 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update OpenTalk outlook add-in version v0.3.6-2 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update keycloak version 26.6.3 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update postgresql version 17.10 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update traefik version v3.7.7 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update garage version v2.3.0 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update mailcrab version v1.7.0 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)
- (deps) Update NDT server version v0.25.3 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)

### ⚙ Miscellaneous

- (docs) Update CHANGELOG (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/190)

## [26.1.0-beta.6] - 2026-05-27

[26.1.0-beta.6]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.6...v26.1.0-beta.6

### 🐛 Bug fixes

- (docs) Add garage and s3_migrate sections, fix duplicate minio_s3_api_port entry

### 📦 Dependencies

- (deps) Update OpenTalk Product version v26.1.0-beta.6 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update OpenTalk Controller version 0.33.4 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update OpenTalk Outlook Add-In version v0.3.6-1 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update Keycloak to version v26.5.7 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))
- (deps) Update RabbitMQ to version v4.2.7 ([!180](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/180))

## [26.1.0-beta.5] - 2026-05-12

[26.1.0-beta.5]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.6...v26.1.0-beta.5

### 🚀 New features

- (ansible) Add bulk user import via CSV/JSON to keycloak ([!157](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/157))
- (web-frontend) Add support for custom video backgrounds via host volume mount ([!160](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/160))
- (traefik) Add csp_allowed_extra_img_src variable to extend img-src in CSP header ([!162](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/162))

### 📦 Dependencies

- (deps) Update dependency ansible-core to v2.20.5 ([!161](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/161))
- (deps) Update dependency community.crypto to v3.2.0 ([!158](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/158))
- (deps) Update dependency community.general to v12.6.0 ([!159](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/159))
- (deps) Update OpenTalk Web Frontend version 2.8.2 ([!169](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/169))
- (deps) Update OpenTalk Controller version 0.33.3 ([!169](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/169))
- (deps) Update OpenTalk Product version 26.1.0-beta.5 ([!169](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/169))

### ⚙ Miscellaneous

- (docs) Add commented production-relevant settings to settings.yml ([!164](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/164))
- (obelisk) Update default values and add missing options ([!163](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/163))
- (docs) Update CHANGELOG ([!169](https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/169))

## [26.1.0-beta.4] - 2026-04-15

[26.1.0-beta.4]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.6...v26.1.0-beta.4

### 🐛 Bug fixes

- (s3-migrate) Migration failed on opensuse (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/155)

### 📦 Dependencies

- (deps) Update OpenTalk Recorder version 0.16.2 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/155)
- (deps) Update OpenTalk Outlook Add-In version 0.3.5 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/155)
- (deps) Update OpenTalk Product version 26.1.0-beta.4 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/155)

### ⚙ Miscellaneous

- (docs) Update CHANGELOG (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/155)
- (ansible) Add missing config options, fix demo user and update config reference (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/155)

## [26.1.0-beta.3] - 2026-04-10

[26.1.0-beta.3]: https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/compare/v25.4.6...v26.1.0-beta.3

### 🚀 New features

- (keycloak) Add custom CA to keycloak truststore (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/146)
- (ansible) Add Garage S3 storage role as default provider (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/136)
- (garage) Integrate Garage as default S3 storage provider (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/136)

### 📦 Dependencies

- (deps) Update dependency community.general to v12.5.0 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/150)
- (deps) Update dependency ansible-core to v2.20.4 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/148)
- (deps) Update dependency community.docker to v5.1.0 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/149)
- (deps) Update dependency community.docker to v5.2.0 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/152)
- (deps) Update OpenTalk Product version v26.1.0-beta.3 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/151)
- (deps) Update OpenTalk Web Frontend version v2.8.1 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/151)
- (deps) Update OpenTalk Controller version v0.33.2 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/151)
- (deps) Add garage version v2.2.0 (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/136)

### ⚙ Miscellaneous

- (docs) Improve TURN and firewall config (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/139)
- (docs) Update CHANGELOG (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/151)
- (obelisk) Set network mode host by default (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/144)
- (ansible) Improve service state handling (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/147)
- (ansible) Cache check_compose_includes result and invalidate on blockinfile change (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/147)
- (ansible) Add project_name in traefik service handler (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/147)
- (ansible) Impove condition to not run when tag is demo-user (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/147)
- (ansible) Short-circuit secrets generation loop when no keys are missing (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/147)
- (packaging) Delete LICENSES folder after package remove (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/147)
- (ci) Var PIPELINE_MERGE_REQUEST_TARGET_BRANCH_NAME added (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/151)
- (docs) Update docs regarding garage migration (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/136)
- (docs) Add garage support for config reference generation (https://git.opentalk.dev/opentalk/opentalk-devops/opentalk-compose/-/merge_requests/136)

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

