# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.35.0] - 2026-09-10

[0.35.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.34.0...v0.35.0

### 🚀 New features

- Close rooms when they are deleted over the REST API ([!2639](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2639), [#1461](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1461))
- (db) Drop casbin rule table ([!2535](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2535), [#1363](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1363))
- (roomserver) Ignore display name in start room requests ([!2618](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2618))
- (auth) Wrap public endpoints with authorization middleware ([!2627](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2627), [#1451](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1451))
- Prepare `GET /rooms/{room_id}/event` for invite code removal ([!2698](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2698), [#1467](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1467))
- (roomserver) Add fields to call-in info ([!2727](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2727), [#1469](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1469))
- (logging) Record auth and authZ process ([!2731](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2731))
- Add optional alias to rooms ([!2723](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2723), [#1457](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1457), [#1458](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1458))
- Add `POST /rooms/name/verify` endpoint ([!2723](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2723), [#1456](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1456), [#1470](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1470))
- (livekit-proxy) Keep the `v1` path component ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- Remove invite-code api ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654), [#1459](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1459))
- One-link authorizer (no invite-code) ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- Ignore invalid authentication tokens ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- Add dev command to query invite links ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- (metrics) Expose metrics for internal roomserver ([!2797](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2797), [#1501](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1501))

### 🐛 Bug fixes

- (jobs) Room deletion is attempted twice when deleting events ([!2639](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2639))
- (test) `sync_files_and_set_missing_to_zero` checks for the wrong asset count ([!2639](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2639))
- (jobs) Events and rooms of deleted users don't get cleaned up ([!2641](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2641), [#1463](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1463))
- (mailer) Mark messages as persistent ([!2669](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2669))
- (api-authorization) Reject invite code access to event endpoints ([!2696](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2696))
- (storage) Preserve raw query string in proxy download token to prevent 403 on strict S3 backends ([!2695](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2695), [#1468](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1468))
- (api-authorization) Don't panic when failing to parse path pattern ([!2724](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2724), [#1482](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1482))
- (authorization) Adjust access to `rooms/{room_id}/invites` ([!2734](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2734), [#1485](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1485))
- (docs) Add missing `/signaling/{token}` endpoint ([!2713](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2713))
- (rooms) Make password_required role-aware ([!2748](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2748))
- (docs) Fix typo ([!2723](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2723))
- (api-actix-web) Typo in variable name ([!2723](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2723))
- (rooms) Fix start pattern const ([!2751](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2751))
- (justfile) Only consider branch local tags when generating changelog ([!2752](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2752))
- (ci) Rename Ubuntu based image to resolute ([!2759](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2759))
- (livekit-proxy) Use GET for rtc/validate endpoint ([!2771](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2771), [#1489](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1489))
- (docs) Fix typos in keycloak docs ([!2773](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2773))
- (docs) Document correct room name suffix max length ([!2775](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2775))
- (authorization) Accept room alias for `/v1/rooms/{room_id_or_alias}/assets/{asset_id}/download` ([!2783](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2783), [#1492](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1492))
- (meeting_details) Patch room with show_meeting_details ([!2793](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2793), [#1495](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1495))

### ⚡ Performance

- (jobs) Avoid redundant event lookups during event deletion ([!2641](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2641))
- Don't clone room id or alias ([!2758](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2758))

### 📚 Documentation

- (test-util) Remove broken link ([!2535](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2535))
- (auth) Update authorization middleware docs ([!2665](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2665), [#1362](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1362))
- (api) Document the correct response body for `GET /events/{event_id}/invites` ([!2693](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2693))
- (api) Remove invite code from `GET /rooms/{room_id}` ([!2696](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2696))
- Document all roomserver modules in the example config ([!2762](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2762))
- Document keycloak token introspection requirements ([!2773](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2773))
- Remove invite code authentication from open-api spec ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- Enable more signaling modules in example config ([!2781](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2781))
- Split migration guide into multiple files ([!2782](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2782))
- Add migration guides for v0.31..v0.34 ([!2782](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2782))
- Don't explain `force_path_style` with the same words ([!2782](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2782))
- Update stale metrics docs ([!2797](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2797))
- Add metrics migration guide for v0.35.0 ([!2797](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2797))
- Add migration guide for 0.35 ([!2809](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2809), [#1497](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1497))

### 🔨 Refactor

- Use expect instead of allow to prevent unused allows ([!2667](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2667))
- (docs) Remove unnecessary utoipa schemas ([!2713](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2713))
- Simplify room checks in `PATCH /events/{event_id}` ([!2742](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2742))

### 📦 Dependencies

- (deps) Document RUSTSEC-2026-0258 ([!2694](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2694))
- (deps) Lock file maintenance ([!2664](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2664), [!2682](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2682), [!2702](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2702), [!2720](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2720), [!2740](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2740), [!2768](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2768), [!2792](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2792))
- (deps) Panic when starting a room due to version mismatch of jsonwebtoken ([!2697](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2697))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag ([!2658](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2658))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag ([!2778](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2778))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.97.1 ([!2684](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2684))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.97.1 ([!2760](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2760))
- (deps) Update h2 to 0.4.16 ([!2694](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2694))
- (deps) Update opentalk ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- (deps) Update opentalk ([!2673](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2673))
- (deps) Update opentalk to 0.1.0 ([!2648](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2648))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.26.0 ([!2644](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2644))
- (deps) Update pre-commit hook davidanson/markdownlint-cli2 to v0.23.0 ([!2649](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2649))
- (deps) Update pre-commit hook davidanson/markdownlint-cli2 to v0.23.1 ([!2676](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2676))
- (deps) Update pre-commit hook davidanson/markdownlint-cli2 to v0.23.2 ([!2703](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2703))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.20.2 ([!2660](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2660))
- (deps) Update redocly/cli docker tag to v1.34.17 ([!2463](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2463))
- (deps) Update redocly/cli docker tag to v1.34.19 ([!2718](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2718))
- (deps) Update rust crate aws-sdk-s3 to v1.138.0 ([!2653](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2653))
- (deps) Update rust crate aws-sdk-s3 to v1.139.0 ([!2690](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2690))
- (deps) Update rust crate base64 to 0.23 ([!2694](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2694))
- (deps) Update rust crate bytes to v1.12.1 ([!2651](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2651))
- (deps) Update rust crate diesel to v2.3.11 ([!2661](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2661))
- (deps) Update rust crate etcd-client to 0.19 ([!2121](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2121))
- (deps) Update rust crate etcd-client to 0.20 ([!2777](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2777))
- (deps) Update rust crate flume to 0.12 ([!2445](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2445))
- (deps) Update rust crate glob to v0.3.4 ([!2688](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2688))
- (deps) Update rust crate itertools to 0.15 ([!2648](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2648))
- (deps) Update rust crate md5 to v0.8.1 ([!2656](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2656))
- (deps) Update rust crate mockall to 0.15 ([!2645](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2645))
- (deps) Update rust crate pdf-extract to 0.12 ([!2650](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2650))
- (deps) Update rust crate redis to v1.5.0 ([!2712](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2712))
- (deps) Update rust crate rstest to 0.27.0 ([!2791](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2791))
- (deps) Update rust crate rustls to v0.23.43 ([!2710](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2710))
- (deps) Update rust crate rustls-pki-types to v1.15.1 ([!2692](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2692))
- (deps) Update rust crate serial_test to v4 ([!2701](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2701))
- (deps) Update rust crate snafu to v0.9.2 ([!2689](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2689))
- (deps) Update rust crate sysinfo to v0.39.6 ([!2657](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2657))
- (deps) Update rust crate tabled to 0.22 ([!2788](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2788))
- (deps) Update rust crate tokio-stream to v0.1.19 ([!2691](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2691))
- (deps) Update rust crate tokio-util to v0.7.19 ([!2687](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2687))
- (deps) Update rust crate toml to v1.1.4 ([!2705](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2705))
- (deps) Update rust crate yaml-rust2 to 0.12.0 ([!2757](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2757))
- (deps) Update static postgres lib to v17.9 ([!2619](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2619))
- (deps) Update stoplight/spectral docker tag to v6.16 ([!2674](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2674))
- (deps) Use vendored swagger ui ([!2759](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2759))

### ⚙ Miscellaneous

- Remove linting denies ([!2667](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2667))
- Minor doc comment fixes ([!2667](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2667))
- (settings) Fix `dead_code` warning when compiling without tests ([!2744](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2744))
- (ci) Use debian trixie for building ubuntu based image ([!2759](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2759))
- Remove dead code ([!2761](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2761))
- Remove unused invite-code errors ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- Remove invite-code parsing from auth middleware ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- Remove unused invite code inventory functions ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- Responde with 401 if accessing unauthenticated ([!2654](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2654))
- Remove legacy and unsupported `stun` section from example ([!2782](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2782))

### Ci

- (pre-commit) Switch from taplo to olpat (taplo is unmaintained) ([!2655](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2655))
- Use container scanning template ([!2634](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2634))
- Remove unused redis container ([!2744](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2744))
- Remove unnecessary commands from test script ([!2744](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2744))

### Test

- (proxy-download) Verify that the query is forwarded accurately ([!2695](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2695))
- Migrate database tests to use testcontainers ([!2744](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2744))
- Migrate tests using S3 to testcontainers ([!2744](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2744))
- Reduce asset count of `sync_files_and_delete_missing` to 99 ([!2744](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2744))

## [0.34.5] - 2026-09-09

[0.34.5]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.34.4...v0.34.5

### 📚 Documentation

- (auth) Update authorization middleware docs ([!2804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2804), [#1362](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1362))
- Enable more signaling modules in example config ([!2804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2804))
- Split migration guide into multiple files ([!2804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2804))
- Add migration guides for v0.31..v0.34 ([!2804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2804))
- Don't explain `force_path_style` with the same words ([!2804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2804))

### 📦 Dependencies

- (deps) Update roomserver to v0.1.4 ([!2804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2804))
- (deps) Update chacha20 ([!2804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2804))

### ⚙ Miscellaneous

- Remove legacy and unsupported `stun` section from example ([!2804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2804))

## [0.34.4] - 2026-08-26

[0.34.4]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.34.3...v0.34.4

### 🐛 Bug fixes

- (livekit-proxy) Use GET for rtc/validate endpoint ([!2772](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2772), [#1490](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1490))

## [0.34.3] - 2026-08-24

[0.34.3]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.34.2...v0.34.3

### 📚 Documentation

- Document all roomserver modules in the example config ([!2763](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2763))

### 📦 Dependencies

- (deps) Update roomserver to 0.1.2 ([!2769](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2769))
- (deps) Lock file maintenance ([!2769](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2769))

## [0.34.2] - 2026-08-14

[0.34.2]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.34.1...v0.34.2

### 🐛 Bug fixes

- (mailer) Mark messages as persistent ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753))
- (storage) Preserve raw query string in proxy download token to prevent 403 on strict S3 backends ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753), [#1479](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1479))
- (api-authorization) Reject invite code access to event endpoints ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753), [#1475](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1475))
- (api-authorization) Don't panic when failing to parse path pattern ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753), [#1483](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1483))
- (authorization) Adjust access to `rooms/{room_id}/invites` ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753), [#1486](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1486))
- (call-in) Re-enable phone number mapping and masking ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753), [#1487](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1487))
- (justfile) Only consider branch local tags when generating changelog ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753))

### 📦 Dependencies

- (deps) Lock file maintenance ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753))

### ⚙ Miscellaneous

- Fix clippy lints ([!2753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2753))

## [0.34.1] - 2026-07-07

[0.34.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.34.0...v0.34.1

### 📦 Dependencies

- (deps) Update rust crate itertools to 0.15 ([!2647](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2647))
- (deps) Update opentalk to 0.1.0 ([!2647](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2647))

### Ci

- (just) Create release branch ([!2647](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2647))
- (just) Tag release and create gitlab release ([!2647](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2647))

## [0.34.0] - 2026-07-07

[0.34.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.33.6...v0.34.0

### 🚀 New features

- (db) Add `invites` table name to `Invite` struct ([!2311](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2311))
- Add example data for `User` ([!2295](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2295))
- Add example data for `Tenant` ([!2295](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2295))
- (db) Add table name to job table structs ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Add rooms table name to `Room` table struct ([!2336](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2336))
- (api) Use updated `PatchEventBody` type ([!2351](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2351))
- (db) Add diesel table name to `SipConfig` table struct ([!2346](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2346))
- Add internal roomserver call-in endpoint ([!1733](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1733))
- Add internal roomserver recording endpoints ([!1733](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1733))
- (db) Add log messages to `TryFrom<RoomStreamingTarget>` ([!2348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2348))
- (db) Add table name to tariff table struct ([!2369](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2369))
- (db) Add diesel table name to `Tenant` table struct ([!2384](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2384))
- (oidc) Convert `OidcCacheError` into `CaptureApiError` ([!2374](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2374), [#1223](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1223))
- (oidc) Oidc cache methods return `CaptureApiError` ([!2374](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2374), [#1223](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1223))
- (oidc) Add internal method for caching errors ([!2374](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2374), [#1223](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1223))
- (oidc) Use `cache_access_token_error` method ([!2374](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2374), [#1223](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1223))
- (oidc) Cache token verification errors properly ([!2374](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2374), [#1223](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1223))
- (oidc) Flatten `get_access_token` return type ([!2374](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2374), [#1223](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1223))
- (db) Add table name to user table struct ([!2400](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2400))
- (signaling) Remove built-in signaling from controller as it will be handled by the roomserver ([!2315](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2315), [#1334](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1334))
- (db) Add table name to module resource table struct ([!2407](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2407))
- Support more S3 providers ([!2409](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2409), [#1347](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1347))
- (roomserver) Update roomserver storage quota on changes ([!2312](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2312), [#1272](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1272))
- (db) Split event date fields into dedicated `event_dates` table ([!2422](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2422))
- (db) Adapt event queries to joined event record model ([!2422](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2422))
- (authorization) Create opentalk-controller-api-authorization crate ([!1310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1310))
- (authorization) Define the initial API interface for the authorizer ([!1310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1310))
- (authorization) Introduce new auth implementation ([!1310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1310))
- (authorization) Add synchronization crate ([!1310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1310))
- (authorization) Implement basic synchronization interface ([!1310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1310))
- (authorization) Add some authorization tests ([!1310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1310))
- Make roomserver configuration mandatory ([!2436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2436), [#1355](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1355))
- (db) Move event recurrence related fields into seperate table ([!2448](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2448))
- (db) Adapt queries to new table structure ([!2448](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2448), [#1154](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1154))
- (guest_access) Extend db types and schema ([!2415](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2415), [#1324](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1324))
- (guest_access) Add migration for adding `guest_access` ([!2415](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2415), [#1324](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1324))
- (guest_access) Extend inventory types ([!2415](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2415), [#1324](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1324))
- (guest_access) Add guest access to the room and event services ([!2415](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2415), [#1324](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1324))
- (guest_access) Add guest access to room API's ([!2415](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2415), [#1324](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1324))
- (guest_access) Check guest access availability in tariff ([!2415](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2415), [#1324](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1324))
- (guest_access) Response with bad request on constraint violation ([!2415](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2415), [#1324](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1324))
- Remove obsolete recorder types ([!2451](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2451))
- Print warning when encountering unknown fields in configuration file ([!2470](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2470))
- (roomserver) Make the `livekit` and `echo` modules mandatory ([!2450](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2450), [#1356](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1356))
- Remove obsolete signaling metrics ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete `SignalingRoomId` ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete `AnyStream` ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete signaling exchange message ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete signaling storage ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete room locking ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete expiring data types ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete `Participant` enum ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete `r3dlock` crate ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- Remove obsolete `module_tester` feature ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- (api) Deprecate `/rooms/{room_id}/roomserver/*` endpoints ([!2475](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2475), [#1372](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1372))
- (roomserver) Add `room_idle_timeout` setting ([!2462](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2462))
- Add internal roomserver implementation ([!2462](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2462))
- (api) Add transcription start endpoint ([!2478](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2478))
- Add signaling endpoint for internal roomserver ([!2506](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2506))
- (roomserver) Add internal asset storage provider ([!2507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2507))
- (roomserver) Add internal module resource provider ([!2507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2507))
- (roomserver) Add internal storage notifier ([!2507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2507))
- (cors) Expose LINK header in CORS configuration ([!2519](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2519))
- (roomserver) Infer the public url from the request in case of an internal roomserver ([!2523](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2523), [#1395](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1395))
- (roomserver) Add livekit proxy for internal roomserver ([!2517](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2517), [#1354](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1354))
- (start) `/start` API handles invite codes ([!2543](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2543), [#1319](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1319))
- (keycloak) Adapt to optional name for keycloak users ([!2516](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2516), [#1393](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1393))
- (roomserver) Pass guest access to roomserver ([!2529](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2529))
- (roomserver) Enforce disabled guest access for built-in roomserver ([!2529](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2529))
- (call-in) Disable call-in when guest access is disabled ([!2620](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2620), [#1450](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1450))
- (auth) Add authorization database backend ([!2602](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2602))
- Add opencloud shared folder support ([!2621](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2621), [#1435](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1435))

### 🐛 Bug fixes

- (ci) Properly pass gitlab token to git cliff subshell ([!2322](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2322))
- (docs) Document that the controller must be restarted after `fix-acl` ([!2323](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2323), [#1298](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1298))
- (livekit-proxy) Consider whisper groups when checking participant's room state ([!2326](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2326))
- (assets) Handle headers with non-ASCII characters ([!2333](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2333))
- (polls) Enable finish polls without votes ([!2335](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2335), [#1152](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1152))
- (livekit-proxy) Add missing GET rtc/validate & rtc/v1/validate endpoint ([!2332](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2332))
- (docs) Fix oidc migration authority option naming ([!2355](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2355))
- (db) Removed unused sip configs module ([!2375](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2375))
- (api) Return 403 if room event belongs to a disabled user ([!2386](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2386), [#1330](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1330))
- Prevent patch_me from overriding missing fields ([!2396](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2396), [#1333](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1333))
- (init) Install default crypto provider for jsonwebtoken crate during initialization ([!2392](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2392), [#1326](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1326))
- (ci) Set required target branch variable for `trigger_changelog_preview` job ([!2406](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2406))
- (docker) Build exactly the opentalk-controller package ([!2420](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2420))
- (api) Calculate the correct ends at for recurring events ([!2427](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2427))
- Restrict logging of errors returned by low-level functions ([!2424](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2424))
- Pass e2e encryption setting to RoomServer ([!2430](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2430), [#1352](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1352))
- Fallback avatar url was not calculated correctly ([!2444](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2444), [#1358](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1358))
- (auth) Properly pass user id from oidc auth middleware to permissions middleware ([!2447](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2447), [#1364](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1364))
- (roomserver) Pass moderator role of non room owners correctly to roomserver ([!2453](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2453), [#1353](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1353))
- Repair broken storage notifier injection ([!2457](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2457))
- (controller-service) Remove legacy service signaling endpoints & re-add removed mod.rs ([!2449](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2449))
- Replace request bodies for internal start endpoints ([!2449](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2449))
- (auth) Load users, invite codes and event invitations at startup ([!2466](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2466))
- (guest_access) Db migration of rooms with enabled waiting room ([!2468](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2468), [#1370](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1370))
- Check upload file size ([!2465](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2465))
- Remove left over `websocket_rate_limit` setting ([!2470](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2470))
- (guest-access) Use waiting room as the default ([!2477](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2477))
- Omit recurring events for time independent queries ([!2485](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2485), [#1376](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1376))
- (db) Move `duration_secs` into `event_dates` table ([!2483](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2483), [#1375](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1375))
- (db) Set default duration_secs to 0 for nonrecurring events ([!2489](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2489))
- (roomserver) Pass configured room idle timeout to in `RoomParameters` ([!2504](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2504))
- Downgrade shutdown signal error to debug level ([!2506](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2506))
- Prevent invite code generation/access when guest access is disabled ([!2528](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2528), [#1396](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1396))
- (auth) Active user with empty name must be able to make requests ([!2516](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2516), [#1393](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1393))
- Only emit pagination cursor when more results exist ([!2562](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2562), [#1414](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1414))
- Enforce guest access in auth middleware ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548), [#1400](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1400), [#1398](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1398))
- (api-authorization) Prevent guest access to e2ee rooms in auth middleware ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548), [#1400](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1400))
- (api-authorization) Prevent guest access with expired invite code in auth middleware ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548), [#1400](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1400))
- (api-authorization) Enforce guest allowed tariff feature in auth middleware ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548), [#1400](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1400))
- Enforce all guest access properties where the guest feature is enforced ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548), [#1398](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1398))
- Enforce all guest access properties in `/v1/rooms/{room_id}/start_invited` ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548))
- (api-authorization) Correct authorizer name in `Debug` implementation ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548))
- (api) Document invite code authentication for `/v1/rooms/{room_id}` endpoint ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548))
- (api-authorization-database) Typo in log message ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548))
- Do not assume the current user to be the room owner when modifying a room or event ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548))
- Do not reject PATCH `/v1/rooms/{room_id}` when guest access is unchanged ([!2568](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2568))
- Update outdated migration guide ([!2612](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2612))
- (settings) Remove unused authorization option ([!2602](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2602))
- Prevent events/rooms with guest access and e2ee enabled from being created ([!2623](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2623), [#1422](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1422))
- Enforce invite validity when building room parameters and event info ([!2630](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2630), [#1453](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1453))
- (auth) Peek request body to extract invite code ([!2635](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2635), [#1455](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1455))
- Actually skip empty paths when deleting shared folders ([!2621](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2621))

### 📚 Documentation

- Update container image build description to new Dockerfile paths ([!2390](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2390))
- (db) Update generated documentation ([!2422](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2422))
- (db) Update generated documentation ([!2448](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2448))
- Update crates in README.md ([!2471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2471))
- (api) Update openapi spec ([!2481](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2481))
- (api) Update openapi spec ([!2475](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2475))
- (api) Remove deprecated section describing roomserver behavior ([!2475](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2475))
- (db) Update generated documentation ([!2483](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2483))
- (roomserver) Add documentation for internal roomserver ([!2505](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2505), [#1380](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1380))
- (start) Update openapi components ([!2547](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2547), [#1406](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1406))
- (tariffs) Document tariff configuration and external assignment ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548))
- (tariffs) Document security implications of modifying disabled features and tariff assignments ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548))
- Document back-channel logout ([!2587](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2587), [#1426](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1426))
- Remove reference to outdated/removed `fix-acl` command ([!2612](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2612))
- (auth) Mark ACL documentation as deprecated ([!2602](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2602))

### 🔨 Refactor

- (db) Move invite table structs into tables module ([!2311](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2311))
- (db) Move invite queries into quries module ([!2311](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2311))
- (db) Rename `SerialId` to `SerialJobId` ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Move job table structs into tables module ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Move job execution table structs into tables module ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Move job execution log table structs into tables module ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Move job queries into queries module ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Move job execution queries into queries module ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Move job execution log queries into queries module ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Remove unused `create_job_execution_log` query ([!2330](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2330))
- (db) Move room table structs into tables module ([!2336](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2336))
- (db) Move room queries into queries module ([!2336](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2336))
- (db) Remove unused delete function ([!2336](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2336))
- (db) Move sip config table structs into tables module ([!2346](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2346))
- (db) Move sip config queries into queries module ([!2346](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2346))
- (db) Remove unused `get` function ([!2346](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2346))
- (db) Remove unused `delete` function ([!2346](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2346))
- (db) Add const `ATTEMPTS` to `create_room_sip_config` ([!2346](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2346))
- (db) Move room streaming target table structs into tables module ([!2348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2348))
- (db) Rename `RoomStreamingTargetRecord` to `RoomStreamingTarget` ([!2348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2348))
- (db) Rename `RoomStreamingTargetNew` to `NewRoomStreamingTarget` ([!2348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2348))
- (db) Move room streaming targets queries into queries module ([!2348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2348))
- (db) Remove unnecessary typed `get_room_streaming_targets` impl ([!2348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2348))
- (db) Move tariff table structs into tables module ([!2369](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2369))
- (db) Move tariff queries into queries moduel ([!2369](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2369))
- (db) Move tenant table structs into tables module ([!2384](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2384))
- (db) Move tenant queries into queries module ([!2384](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2384))
- (db) Remove unused tenant queries ([!2384](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2384))
- (db) Move user table structs into tables module ([!2400](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2400))
- (db) Move user queries into queries module ([!2400](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2400))
- (db) Move `newtypes::Duration` into tables module ([!2402](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2402))
- (db) Move `newtypes::LanguageIdentifier` into `tables::users::LanguageIdentifier` ([!2402](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2402), [#1344](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1344))
- (db) Move module resource table struct into tables module ([!2407](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2407))
- (db) Move module resource queries into queries module ([!2407](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2407))
- (db) Move module resource tests into tests ([!2407](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2407))
- (db-storage) Return file size from delete query ([!2312](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2312))
- (signaling-core) Use `Quota` type for `verify_storage_usage` ([!2312](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2312))
- (db) Move `UpdateEventException` query into queries module ([!2401](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2401))
- (db) Move `NewEventException` query into queries module ([!2401](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2401))
- (db) Rename `cursor` module to `types` ([!2422](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2422))
- (oidc) Move `check_access_token`  to a dedicated module ([!2327](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2327), [#1301](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1301))
- (oidc) Rename `provision_user` function and move to a dedicated module ([!2327](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2327), [#1301](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1301))
- (oidc) Rename `authenticate_user` for clarity ([!2327](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2327), [#1301](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1301))
- (settings) Remove obsolete websocket rate limit ([!2436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2436))
- (settings) Remove obsolete screen share permissions from defaults ([!2436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2436))
- (settings) Remove obsolete etherpad settings ([!2436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2436))
- (settings) Remove obsolete spacedeck settings ([!2436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2436))
- (settings) Remove obsolete livekit settings ([!2436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2436))
- (settings) Remove obsolete subroom audio settings ([!2436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2436))
- Use explicit destructuring to detect unused settings ([!2470](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2470))
- Don't use 'reports' settings type for deprecation warning ([!2470](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2470))
- Log legacy fields using serde_ingored instead of `Extensions` ([!2470](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2470))
- (settings) Remove obsolete `SettingsError::LiveKitMissing` ([!2450](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2450))
- Separate roomserver signaling from REST API ([!2462](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2462))
- Remove 'roomserver' affixes from signaling functions ([!2462](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2462))
- Abstract roomserver implementation behind a `RoomServerBackend` trait ([!2462](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2462))
- Explicitly match all client error variants in `ExternalRoomServer` ([!2462](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2462))
- Move asset storage related modules from `opentalk-signaling-core` to a dedicated crate ([!2499](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2499))
- Move exchange task from `opentalk-signaling-core` to `opentalk-controller-core` crate ([!2499](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2499))
- Move redis wrapper from `opentalk-signaling-core` to `opentalk-controller-service` crate ([!2499](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2499), [#1341](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1341))
- (core) Require `Debug` for `StorageNotifier` trait ([!2507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2507))
- (roomserver) Rename `RoomServerStorageNotifier` to `ExternalStorageNotifier` ([!2507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2507))
- (roomserver) Initialize `ModuleRegistry` only once when using the internal roomserver ([!2507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2507))
- (roomserver) Move `Send` + `Sync` bounds from trait objects to trait definitions ([!2507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2507))
- (roomserver) Use `log` instead of `tracing` in external roomserver ([!2517](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2517))
- (api-actix-web) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (api-authorization-database) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (api-authorization-synchronization) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (api-authorization) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (settings) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (asset-storage) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (database) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (db-storage) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (core) Switch from log to tracing crate ([!2524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2524))
- (deps) Use opentalk-types-api-common wherever applicable ([!2551](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2551))
- Add utility function for checking if call-in is allowed ([!2553](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2553))
- Remove invite code validation from `/v1/room/{room_id}/tariff` ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548), [#1400](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1400))
- Remove invite code validation from `/v1/room/{room_id}/event` ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548), [#1400](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1400))
- (inventory) Reorder parameters in `is_call_in_allowed()` ([!2568](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2568))
- (auth) Rename `opentalk-controller-api-authorization-{database,memory}` ([!2602](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2602))
- (settings) Gate test helper behind `test-util` feature ([!2629](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2629))
- (auth) Remove in-memory authorizer ([!2629](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2629), [#1452](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1452))

### 📦 Dependencies

- (deps) Address cargo deny findings ([!2376](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2376))
- (deps) Document RUSTSEC-2026-0099 ([!2437](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2437))
- (deps) Document RUSTSEC-2026-0104 ([!2421](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2421))
- (deps) Document RUSTSEC-2026-0173 ([!2580](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2580))
- (deps) Document RUSTSEC-2026-0192 ([!2628](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2628))
- (deps) Document RUSTSEC-2026-0194 and RUSTSEC-2026-0195 ([!2638](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2638))
- (deps) Explicitly opt in to `aws-lc-rs` crypto backend ([!2363](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2363), [#1320](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1320))
- (deps) Lock file maintenance ([!2307](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2307), [!2373](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2373), [!2454](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2454), [!2484](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2484), [!2496](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2496), [!2511](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2511), [!2522](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2522), [!2542](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2542), [!2567](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2567), [!2435](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2435))
- (deps) Move `ring` to dev-dependencies ([!2365](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2365))
- (deps) Remove no longer present RUSTSEC-2026-0049 from cargo-deny allow-list ([!2389](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2389))
- (deps) Remove obsolete opentalk-etherpad-client dependency ([!2436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2436))
- (deps) Remove obsolete signaling types deps ([!2410](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2410))
- (deps) Remove unmatched skip crates ([!2415](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2415))
- (deps) Skip deny hashbrown and indexmap ([!2431](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2431))
- (deps) Update alpine docker tag to v3.24 ([!2583](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2583))
- (deps) Update alpine/git docker tag to v2.54.0 ([!2590](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2590))
- (deps) Update crate crossbeam-epoch to v0.9.20 ([!2642](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2642))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag ([!2455](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2455))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag ([!2557](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2557))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.1 ([!2426](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2426))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.1 ([!2428](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2428))
- (deps) Update opentalk ([!2412](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2412))
- (deps) Update opentalk ([!2421](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2421))
- (deps) Update opentalk ([!2481](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2481))
- (deps) Update opentalk ([!2507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2507))
- (deps) Update opentalk and sysinfo ([!2462](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2462))
- (deps) Update opentalk to 0.0.28 ([!2359](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2359))
- (deps) Update opentalk to 0.0.31 ([!2394](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2394))
- (deps) Update opentalk-roomserver crates to 0.0.27 ([!2354](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2354), [#1318](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1318))
- (deps) Update opentalk-roomserver to 0.0.25 ([!2331](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2331))
- (deps) Update opentalk-types-api-v1 to 0.59.0 ([!2549](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2549))
- (deps) Update opentalk-types-common to 0.44.1 ([!2548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2548))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.25.0 ([!2492](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2492))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.9 ([!2597](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2597))
- (deps) Update pre-commit hook markdownlint/markdownlint to v0.17.0 ([!2573](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2573))
- (deps) Update redocly/cli docker tag to v1.34.11 ([!2352](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2352))
- (deps) Update registry.gitlab.com/pipeline-components/markdownlint docker tag to v0.14.15 ([!2540](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2540))
- (deps) Update roomserver dependencies to v0.0.26 ([!2351](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2351))
- (deps) Update rust crate `tokio-postgres` to 0.7.18 ([!2591](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2591))
- (deps) Update rust crate arc-swap to v1.9.1 ([!2413](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2413))
- (deps) Update rust crate aws-sdk-s3 to v1.135.0 ([!2576](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2576))
- (deps) Update rust crate axum to v0.8.9 ([!2433](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2433))
- (deps) Update rust crate chrono to v0.4.45 ([!2575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2575))
- (deps) Update rust crate clap to v4.6.0 ([!2337](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2337))
- (deps) Update rust crate config to v0.15.21 ([!2339](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2339))
- (deps) Update rust crate diesel to v2.3.10 ([!2578](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2578))
- (deps) Update rust crate diesel to v2.3.8 ([!2480](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2480))
- (deps) Update rust crate diesel-async to 0.9, deadpool-runtime to 0.3 and metrics to 0.24.6 ([!2368](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2368))
- (deps) Update rust crate diesel-async to v0.9.1 ([!2579](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2579))
- (deps) Update rust crate either to v1.16.0 ([!2531](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2531))
- (deps) Update rust crate env_logger to v0.11.10 ([!2382](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2382))
- (deps) Update rust crate http to v1.4.2 ([!2581](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2581))
- (deps) Update rust crate http-request-derive to 0.6.0 ([!2425](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2425))
- (deps) Update rust crate insta to v1.47.2 ([!2388](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2388))
- (deps) Update rust crate livekit-api to v0.4.24 ([!2526](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2526))
- (deps) Update rust crate log to v0.4.32 ([!2574](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2574))
- (deps) Update rust crate rand to v0.10.1 ([!2431](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2431))
- (deps) Update rust crate redis to v1.2.2 ([!2556](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2556))
- (deps) Update rust crate rustc-hash to v2.1.2 ([!2398](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2398))
- (deps) Update rust crate serde_with to v3.21.0 ([!2577](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2577))
- (deps) Update rust crate serial_test to v3.5.0 ([!2565](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2565))
- (deps) Update rust crate snafu to v0.9.1 ([!2563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2563))
- (deps) Update rust crate sysinfo to v0.39.3 ([!2555](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2555))
- (deps) Update rust crate tabled to 0.21 ([!2566](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2566))
- (deps) Update rust crate tempfile to v3.27.0 ([!2328](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2328))
- (deps) Update rust crate toml to v1.1.2 ([!2408](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2408))
- (deps) Update rust crate tracing-opentelemetry to 0.33.0 ([!2041](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2041))
- (deps) Update rust crate tracing-subscriber to v0.3.23 ([!2345](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2345))
- (deps) Update rust crate utoipa to v5.5.0 ([!2498](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2498))
- (deps) Update rust crate uuid to v1.23.3 ([!2582](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2582))
- (deps) Update ubuntu docker tag to v26 ([!2479](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2479))

### ✨ Style

- Indent note when printing obsolete settings warning ([!2470](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2470))
- Remove trailing whitespaces ([!2621](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2621))

### ⚙ Miscellaneous

- (logs) Only log once that the internal service route is disabled ([!2357](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2357))
- Fix typos in comments ([!2410](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2410))
- (logs) Don't warn about websocket protocol errors ([!2312](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2312))
- (logs) Debug logs for object upload ([!2312](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2312))
- Remove `opentalk-types-signaling-recording` from deny.toml ([!2450](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2450))
- Whitelist CVE-2026-8376 and CVE-2026-42496 for debian trixy ([!2624](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2624))
- Remove unused opentalk-report-generation crate ([!2637](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2637))
- Remove unused casbin crate ([!2637](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2637))

### Ci

- (just) Update changelog tool to opentalk git-cliff ([!2334](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2334))
- Use template for changelog preview job ([!2459](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2459))
- (pre-commit) Use nightly formatter ([!2470](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2470))
- Build dev images for release branches ([!2536](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2536), [#1379](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1379))
- Use container build template ([!2552](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2552), [#1379](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1379))
- Use rust-lint template ([!2617](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2617))
- Use common lint template ([!2617](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2617))

### Test

- (oidc) Unit tests for the oidc cache ([!2295](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2295))
- (db) Update `get_events` tests for `event_dates` migration ([!2422](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2422))
- (inventory) Add test for `is_call_in_allowed()` ([!2568](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2568))
- (inventory) Remove `println!` from test ([!2620](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2620))

## [0.33.7] - 2026-08-05

[0.33.7]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.33.6...v0.33.7

### 🐛 Bug fixes

- (waiting_room) Enable waiting room on debrief or kick ([!2671](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2671), [#1465](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1465))
- (storage) Preserve raw query string in proxy download token to prevent 403 on strict S3 backends ([!2717](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2717), [#1478](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1478))
- (metrics) Record participant joined after room is created ([!2722](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2722), [#1477](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1477))

### 📚 Documentation

- Fix migration guide headline ([!2614](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2614))

### 📦 Dependencies

- (deps) Lock file maintenance ([!2728](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2728))
- (deps) Update pre-commit hooks ([!2728](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2728))

### ⚙ Miscellaneous

- Fix clippy lints for rustc 1.97.0 ([!2671](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2671))

### Ci

- (pre-commit) Switch from taplo to olpat (taplo is unmaintained) ([!2728](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2728))

## [0.33.6] - 2026-06-18

[0.33.6]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.33.5...v0.33.6

### 🚀 New features

- (keycloak) Adapt to optional name for keycloak users ([!2595](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2595), [#1432](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1432))

### 🐛 Bug fixes

- (auth) Active user with empty name must be able to make requests ([!2595](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2595), [#1432](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1432))

### 📦 Dependencies

- (deps) Update rust crate `tokio-postgres` to 0.7.18 ([!2593](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2593), [#1430](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1430))

### Ci

- Use container build template ([!2606](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2606), [#1379](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1379))
- (just) Don't consider tags from other branches ([!2606](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2606))

## [0.33.5] - 2026-05-29

[0.33.5]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.33.4...v0.33.5

### 🐛 Bug fixes

- (livekit-proxy) Allow popout steams ([!2558](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2558), [#1412](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1412))

## [0.33.4] - 2026-05-26

[0.33.4]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.33.3...v0.33.4

### 🐛 Bug fixes

- More robust hyphenation ([!2527](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2527), [#1394](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1394))
- Ensure tables don't overflow ([!2527](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2527))

### 📦 Dependencies

- (deps) Ignore RUSTSEC-2026-0138 ([!2534](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2534))

### Ci

- Build dev images for release branches ([!2539](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2539), [#1379](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1379))

## [0.33.3] - 2026-05-11

[0.33.3]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.33.2...v0.33.3

### 🐛 Bug fixes

- (legal-vote) Wrap long words in report PDF ([!2467](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2467), [#1368](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1368))
- (moderation) Guest users that joined before the waiting room was enabled do not skip ([!2458](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2458), [#1371](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1371))
- Check upload file size ([!2469](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2469))

### 📦 Dependencies

- (deps) Lock-file maintainance ([!2513](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2513))

### Ci

- Use template for changelog preview job ([!2460](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2460))

## [0.33.2] - 2026-03-28

[0.33.2]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.33.1...v0.33.2

### 🐛 Bug fixes

- (api) Return 403 if room event belongs to a disabled user ([!2395](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2395), [#1330](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1330))
- (init) Install default crypto provider for jsonwebtoken crate during initialization ([!2393](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2393), [#1327](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1327))

## [0.33.1] - 2026-03-23

[0.33.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.33.0...v0.33.1

### 🐛 Bug fixes

- (docs) Document that the controller must be restarted after `fix-acl` ([!2325](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2325), [#1299](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1299))
- (assets) Handle headers with non-ASCII characters ([!2341](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2341), [#1311](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1311))
- (livekit-proxy) Consider whisper groups when checking participant's room state ([!2329](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2329), [#1307](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1307))
- (livekit-proxy) Add missing GET rtc/validate & rtc/v1/validate endpoint ([!2356](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2356))

### 📦 Dependencies

- (deps) Explicitly opt in to `aws-lc-rs` crypto backend ([!2364](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2364), [#1320](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1320))
- (deps) Address cargo deny findings ([!2377](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2377))

### Ci

- (just) Update changelog tool to opentalk git-cliff ([!2378](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2378))

## [0.33.0] - 2026-03-10

[0.33.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.0...v0.33.0

### 🚀 New features

- (cli) Deprecate `--reload` parameter in favor of `reload` subcommand ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973))
- (cli) Health command ([!1963](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1963))
- Include storage quota information when joining the room ([!1957](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1957), [#1161](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1161))
- (roomserver) Add endpoint for RoomServer asset upload ([!1784](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1784), [#1114](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1114))
- (docs) Prepare documentation for mkdocs-material ([!1812](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1812), [#1145](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1145))
- (ci) Load images only if necessary ([!1996](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1996))
- (ci) Push images to new registry ([!1980](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1980), [#1191](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1191))
- (roomserver) Pass language preferences to roomserver when creating a room ([!2024](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2024))
- Conditionally set x-forwarded-host header in oidc introspect and userinfo requests ([!2029](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2029))
- Interweave events and event exceptions at GET /events endpoint ([!2004](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2004))
- Update NewEvent to mirror opentalk-types-api-v1 constraints ([!2056](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2056))
- (ci) Include commit evidence job ([!2073](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2073))
- Update NewEvent to mirror opentalk-types-api-v1 constraints ([!2096](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2096))
- (cache) Implement invalidation ([!2122](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2122))
- (cache) Implement hashing wrapper ([!2142](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2142))
- (cache) Use hashing wrapper in the application ([!2142](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2142))
- (assets) Add endpoint for short lived download URLs ([!2102](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2102), [#1213](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1213))
- Add last_authenticated field ([!2138](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2138), [#1230](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1230))
- (inventory) Enforce date constraints matching opentalk-types ([!2173](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2173))
- (call-in) Phone number masking for call-in participants ([!2166](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2166), [#1231](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1231))
- Interweave events and event instances at GET /events/instances endpoint ([!2160](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2160))
- (oidc) Fetch sub attempt via introspection ([!2157](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2157))
- (oidc) Add jwt access token claims for verificaiton ([!2157](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2157))
- (oidc) Use verification info type ([!2157](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2157))
- (ci) Trigger service integration tests after merge request container build ([!2182](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2182))
- Use opentalk-service-auth for roomserver services ([!2177](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2177))
- (roomserver) Add websocket rate limit ([!2192](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2192))
- (ci) Trigger service integration tests for the default branch ([!2193](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2193))
- (ci) Add release mr creation job ([!2135](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2135))
- (oidc) Implement endpoint for OIDC back channel logout ([!2194](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2194))
- (settings) Don't require signaling settings when room server is configured ([!2199](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2199))
- (roomserver) Restrict CORS based on allowed origins ([!2178](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2178))
- (roomserver) Add internal module resource api ([!1842](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1842))
- (oidc) Make jwt token verification more generic ([!2214](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2214))
- (oidc) Add logout token claims type ([!2214](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2214))
- (oidc) Verify logout token ([!2214](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2214))
- (oidc) Handle post logout request ([!2214](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2214))
- (oidc) Reset ttl on entry update in local cache ([!2262](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2262))
- (cache) Introduce cache update modes ([!2262](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2262))
- (cache) Implement update modes for the local cache ([!2262](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2262))
- (cache) Use `set_options` method for inserting a redis entry ([!2262](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2262))
- (cache) Implement update modes for the redis cache ([!2262](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2262))
- (oidc) Introduce `update_access_token` method ([!2267](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2267))
- (oidc) Update access token on user profile update ([!2267](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2267))
- (ci) Switch to buildah ([!2278](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2278))
- (oidc) Add cache for sub related logout markers ([!2147](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2147))
- (oidc) Enable cache creation with and without key hashing ([!2147](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2147))
- (oidc) Cache logout marker for a sub ([!2147](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2147))
- (oidc) Rename oidc cache error to be more generic ([!2147](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2147))
- (oidc) Implement logout marker type ([!2147](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2147))
- (oidc) Add logout marker to the access tokens cache ([!2266](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2266))
- (oidc) Calculate logout marker for a new valid access token ([!2266](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2266))
- (oidc) Invalidate access tokens after their sub logout ([!2266](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2266))
- (oidc) Reponse unauthorized for a revokde token ([!2266](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2266))
- (roomserver) Patch room parameters when the event title or password change ([!2294](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2294), [#1266](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1266))
- (db) Add `assets` table name to `Asset` struct ([!2297](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2297))
- (db) Add `room_assets` table name to `RoomAsset` struct ([!2297](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2297))

### 🐛 Bug fixes

- Don't return group memberships for groups outside your tenant ([!1988](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1988), [#1187](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1187))
- (l10n) Legal-vote fluent file format ([!2013](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2013))
- (config) Remove etcd section from default config file ([!2014](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2014), [#1197](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1197))
- (config) Allow reading lists from comma-separated environment variables ([!2015](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2015))
- (log) Mention "inventory" instead of "storage" in log messages ([!2049](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2049), [#1201](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1201))
- (api) Correctly communicate 404 for missing database entries instead of 500 ([!2063](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2063), [#1206](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1206))
- (ci) Rules for container tag creation ([!2078](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2078))
- (inventory) Mark time independent events properly ([!2092](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2092), [#1205](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1205))
- (api-v1) Overwrite `shared_folder` with patched state ([!2113](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2113))
- (db) Properly handle users which have no language set ([!2125](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2125), [#1195](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1195))
- Calculate event instances page offset properly ([!2129](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2129))
- Handle single and recurring events corectly ([!2129](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2129))
- (module-resources) Allow empty paths in json operations ([!2139](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2139))
- (training-participation-report) Remove unreachable! statements ([!2172](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2172), [#1241](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1241))
- (e2ee) Verify invites processing ([!2174](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2174))
- (e2ee) Remove guests allowed feature ([!2174](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2174))
- (ci) Pass correct image environment variables to service integration tests ([!2190](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2190))
- (api) Provide `/rooms/{room_id}/assets` POST endpoint handler again after refactoring ([!2189](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2189), [#1248](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1248))
- (roomserver) Modules not initialized in the controller are disabled in the roomserver ([!2175](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2175))
- (livekit) Re-enable `rustls-tls-native-roots` for reqwest 0.12 ([!2201](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2201), [#1251](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1251))
- (ci) Set correct ref for release creation ci template repository ([!2210](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2210))
- (docs) Move `service_api_keys` under the correct section (http) in the example config ([!2198](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2198))
- (docs) Remove asset_storage section from example.toml ([!2218](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2218))
- (oidc) Normalize incoming email ([!2239](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2239))
- (cli) Print updated tariff ([!2243](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2243))
- (tests) Mark test add_to_non_empty() as serial ([!2246](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2246))
- (settings) Parse service_api_keys environment variable as list ([!2242](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2242))
- (ci) Reduce usage of nightly compiler for running tests ([!2259](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2259), [#1260](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1260))
- (oidc) `insert` method of local cache must use cache default ttl ([!2262](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2262))
- (core) Proxy livekit connection to workaround livekit auth limitations ([!2270](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2270), [#1258](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1258))
- (api) Block the creation of invite codes for e2ee events ([!2298](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2298))
- (api) Use default event date when patch request omits date ([!2319](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2319))
- (ci) Properly pass gitlab token to git cliff subshell ([!2322](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2322))

### 📚 Documentation

- Fix spelling and links in report generation docs ([!2034](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2034))
- Add typst documentation to meeting reports ([!2034](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2034))
- Fix broken links in http server docs ([!2036](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2036))
- Remove deprecated janus docs ([!2162](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2162))
- Add chat rate limiting to example config ([!2162](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2162))
- Add roomserver configuration docs ([!2162](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2162))

### 🔨 Refactor

- (cli) Unify subcommand structure ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973))
- (cli) Handle all subcommands inside an `exec(…) method` ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973))
- (cli) Load settings inside subcommand only where required ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973))
- (cli) Move top-level subcommand handling into `exec(…)` method ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973))
- (cli) No longer store full arguments struct where only config path is needed ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973))
- (cli) Load settings in commonly used function which also initializes log output ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973))
- (cli) Encapsulate subcommand execution ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973))
- (cli) Only initialize and start controller struct when required ([!1973](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1973), [#1184](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1184))
- (cli) Move clap Args struct into opentalk-controller ([!1974](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1974))
- Make user theme fields optional & use 'Theme' enum ([!1977](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1977), [#1175](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1175))
- Use struct instead of tuple for `save_asset` return type ([!1784](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1784))
- Return remaining storage quota ([!1784](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1784))
- (oidc) Move OidcContext into a separate module ([!2050](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2050))
- (oidc) Move RealmRoles struct into controller service oidc module ([!2050](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2050))
- (oidc) Move IntrospectInfo into a separate module ([!2050](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2050))
- (oidc) Move OpenIdConnectUserInfo into a separate module ([!2050](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2050))
- (oidc) Add OidcTokenHandler trait ([!2050](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2050))
- (oidc) Add build_oidc_token_handler function ([!2050](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2050))
- (oidc) Replace OidcContext usage by OidcTokenHandler trait ([!2050](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2050))
- (inventory) Provide helper for field access ([!2092](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2092))
- (inventory) Simplify ends_at_of_first_occurence ([!2096](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2096))
- (inventory) Restructure patch_event_instance ([!2096](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2096))
- (inventory) Borrow all values in from_room ([!2096](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2096))
- (inventory) Remove starts_at_of and ends_at_of helper ([!2096](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2096))
- (cache) Move redis cache implementation into separate module ([!2100](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2100))
- (cache) Move local cache implementation into separate module ([!2100](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2100))
- (cache) Introduce CacheStorage trait as the general interface for interacting with caches ([!2100](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2100))
- (cache) Add overlay cache instead of hardcoded optional redis cache ([!2100](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2100))
- (cache) Move key and value type constraints close to the cache implementations ([!2111](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2111))
- (cache) Move encoding and decoding into Value trait ([!2111](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2111))
- (cache) Introduce cacheable variants of datatypes where required ([!2111](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2111))
- (cache) Use rkyv instead of bincode for storing data in cache ([!2111](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2111))
- (cache) Remove bincode dependency ([!2111](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2111))
- (cache) Improve clarity redis cache key by renaming type and field ([!2111](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2111))
- (cache) Improve per entry expiration for local cache ([!2122](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2122))
- (cache) Output key type of the hashing ([!2142](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2142))
- (oidc) Move oidc cache ([!2149](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2149))
- (oidc) Remove unnecessary User prefix ([!2149](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2149))
- (oidc) Extract upserting for patch_me ([!2149](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2149))
- (oidc) Caching of access tokens ([!2149](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2149))
- (oidc) Use const for min token ttl ([!2149](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2149))
- (oidc) Make token insertion as cache methods ([!2149](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2149))
- (oidc) Move cacheable types to oidc ([!2149](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2149))
- (api) Introduce opentalk-controller-api-actix-web crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `.well-known/opentalk/api` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/auth/login` POST into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/auth/login` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}/start_invited` POST into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}/roomserver/start_invited` POST into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/invite/verify` POST into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/turn` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}/assets/{asset_id}/proxy` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/users/find` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/users/me` PATCH into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/users/me` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/users/me/tariff` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/users/me/assets` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/users/{user_id}` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms` POST into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}` PATCH into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}/event` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}/tariff` GET into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}/start` POST into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}/roomserver/start` POST into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}` DELETE into api crate ([!2181](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2181))
- (api) Move `v1/rooms/{room_id}/event` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/auth/login` POST into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/turn` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events` POST into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/instances` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}` PATCH into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}` DELETE into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/users/me/event_favorites/{event_id}` PUT and DELETE into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/instances/{instance_id}` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/instances` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/instances/{instance_id}` PATCH into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/invites` POST into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/invites` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/invites/email` DELETE into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/invites/{user_id}` DELETE into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/invites/email` PATCH into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/invites/{user_id}` PATCH into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/invite` PATCH into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/invite` DELETE into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/shared_folder` GET into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/shared_folder` PUT into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/events/{event_id}/shared_folder` DELETE into api crate ([!2183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2183))
- (api) Move `v1/rooms/{room_id}/sip` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/sip` PUT into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/sip` DELETE into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/invites` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/invites` POST into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/invites/{invite_code}` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/invites/{invite_code}` PUT into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/invites/{invite_code}` DELETE into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/assets` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/assets/{asset_id}` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/assets/{asset_id}/download` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/assets` POST into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/assets/{asset_id}` DELETE into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/streaming_targets` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/streaming_targets/{streaming_target_id}` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/streaming_targets` POST into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/streaming_targets` PATCH into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/rooms/{room_id}/streaming_targets/{streaming_target_id}` DELETE to api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/users/me/pending_invites` GET into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (api) Move `v1/services/call_in/start` POST  into api crate ([!2170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2170))
- (database) Drop redundant `is_time_independent` column ([!2176](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2176))
- (db) Alias `opentalk-inventory` in opentalk-db-storage ([!2202](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2202))
- Move '/services/roomserver' routes to '/internal' ([!2234](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2234))
- (cli) Consume tariff on update ([!2244](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2244))
- (oidc) Use `AccessToken` type for instant update of the user profile ([!2267](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2267))
- (oidc) Update user profile via generic `insert_access_token` ([!2267](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2267))
- (oidc) Move `get_access_token` to the oidc cache ([!2266](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2266))
- (oidc) Extract `calculate_logout_marker` function ([!2266](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2266))
- (oidc) Move `OidcCacheError` to its own mod ([!2266](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2266))
- (db) Prepare tables and queries modules ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move events table structs into tables module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event exceptions table structs into tables module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event invites table structs into tables module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event favorites table structs into tables module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event training participation report table structs into tables module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event email invites table structs into tables module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event shared folders table structs into tables module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Prepare events queries module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event cursor structs into queries module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event queries into queries module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event training participation report queries into queries module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event invite queries into queries module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event favorites into queries module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event exception queries into queries module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Move event email invite queries into queries module ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- (db) Rename `get_email_invites_pending_for_user` to `get_invites_pending_for_user` ([!2225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2225))
- Use total and used instead of remaining quota ([!2299](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2299))
- (db) Move asset table structs into tables module ([!2297](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2297))
- (db) Move asset queries into queries module ([!2297](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2297))
- (db) Move group table structs into tables module ([!2304](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2304))
- (db) Move user group table structs into tables module ([!2304](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2304))
- (db) Move group queries into queries module ([!2304](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2304))

### 📦 Dependencies

- (deps) Update rust crate clap to v4.5.53 ([!1966](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1966))
- (deps) Update rust crate axum to v0.8.7 ([!1964](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1964))
- (deps) Update rust crate aws-sdk-s3 to v1.115.0 ([!1975](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1975))
- (deps) Update rust crate actix-web to v4.12.0 ([!1968](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1968))
- (deps) Update rust crate cargo_metadata to v0.23.1 ([!1965](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1965))
- (deps) Update rust crate serde_with to v3.16.0 ([!1970](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1970))
- (deps) Update rust crate bytes to v1.11.0 ([!1969](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1969))
- (deps) Update rust crate config to v0.15.19 ([!1967](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1967))
- (deps) Update livekit ([!1978](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1978))
- (deps) Update rust crate insta to v1.44.1 ([!1976](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1976))
- (deps) Update opentalk-types ([!1977](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1977))
- (deps) Lock file maintenance ([!1981](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1981))
- (deps) Update rust crate serde_with to v3.16.1 ([!1995](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1995))
- (deps) Update opentalk-types-api-v1 to v0.47.1 to allow invitees_max parameter to be 0 ([!2000](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2000), [#1190](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1190))
- (deps) Lock file maintenance ([!2002](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2002))
- (deps) Update rust crate aws-sdk-s3 to v1.116.0 ([!2008](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2008))
- (deps) Update pre-commit hook markdownlint/markdownlint to v0.15.0 ([!1989](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1989))
- (deps) Update rust crate uuid to v1.19.0 ([!2003](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2003))
- (deps) Update rust crate log to v0.4.29 ([!2009](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2009))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.7 ([!1982](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1982))
- (deps) Update rust crate derive_more to v2.1.0 ([!2007](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2007))
- (deps) Update opentalk-types-api-v1 to v0.48.0 ([!2006](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2006))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.8 ([!2010](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2010))
- (deps) Update alpine docker tag to v3.23 ([!2011](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2011))
- (deps) Update rust crate opentalk-types-common to v0.40.1 ([!2012](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2012))
- (deps) Lock file maintenance ([!2018](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2018))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.9 ([!2021](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2021))
- (deps) Update rust crate aws-sdk-s3 to v1.117.0 ([!2028](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2028))
- (deps) Update diesel-async to 0.7, diesel to 2.3.4 ([!2030](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2030))
- (deps) Update rust crate reqwest to v0.12.25 ([!2025](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2025))
- (deps) Update livekit ([!2023](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2023))
- (deps) Update redocly/cli docker tag to v1.34.6 ([!2027](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2027))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag ([!2033](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2033))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.92.0 ([!2035](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2035))
- (deps) Update rust crate tracing-actix-web to v0.7.20 ([!2037](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2037))
- (deps) Update opentalk-types ([!2032](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2032))
- (deps) Update rust crate redis to v1.0.1 ([!2048](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2048))
- (deps) Lock file maintenance ([!2047](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2047))
- (deps) Update opentalk-types-api-v1 to 0.50.1 ([!2052](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2052))
- (deps) Update rust crate reqwest to v0.12.26 ([!2053](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2053))
- (deps) Update rust crate aws-sdk-s3 to v1.118.0 ([!2054](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2054))
- (deps) Update rust crate yaml-rust2 to 0.11.0 ([!2055](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2055))
- (deps) Update rust crate toml to v0.9.9 ([!2059](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2059))
- (deps) Update rust crate rustls-pki-types to v1.13.2 ([!2058](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2058))
- (deps) Update alpine/git docker tag to v2.52.0 ([!2060](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2060))
- (deps) Update rust crate cidr to v0.3.2 ([!2061](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2061))
- (deps) Update rust crate toml to v0.9.10 ([!2062](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2062))
- (deps) Update rust crate insta to v1.45.0 ([!2066](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2066))
- (deps) Update rust crate diesel to v2.3.5 ([!2067](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2067))
- (deps) Update rust crate tracing to v0.1.44 ([!2064](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2064))
- (deps) Update rust crate moka to v0.12.12 ([!2070](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2070))
- (deps) Update rust crate axum to v0.8.8 ([!2069](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2069))
- (deps) Update rust crate serde_json to v1.0.146 ([!2074](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2074))
- (deps) Update rust crate derive_more to v2.1.1 ([!2072](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2072))
- (deps) Update rust crate reqwest to v0.12.27 ([!2075](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2075))
- (deps) Lock file maintenance ([!2071](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2071))
- (deps) Update rust crate reqwest to v0.12.28 ([!2076](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2076))
- (deps) Update rust crate arc-swap to v1.8.0 ([!2077](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2077))
- (deps) Update pre-commit hook andrejorsula/pre-commit-cargo to v0.5.0 ([!2083](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2083))
- (deps) Lock file maintenance ([!2085](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2085))
- (deps) Lock file maintenance ([!2088](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2088))
- (deps) Update rust crate rustls to v0.23.36 ([!2093](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2093))
- (deps) Update dependency linguify to v0.5.0 ([!2068](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2068))
- (deps) Document accepted RUSTSEC advisories ([!2097](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2097))
- (deps) Update rust crate serde_json to v1.0.149 ([!2086](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2086))
- (deps) Remove RUSTSEC-2026-0002 from allowlist after is no longer encountered ([!2101](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2101))
- (deps) Update openidconnect and jsonwebtoken crates ([!2065](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2065), [#1207](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1207))
- (deps) Lock file maintenance ([!2104](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2104))
- (deps) Update rust crate tracing-actix-web to v0.7.21 ([!2106](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2106))
- (deps) Update pre-commit hook adrienverge/yamllint to v1.38.0 ([!2105](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2105))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.24.0 ([!2107](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2107))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.0 ([!2098](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2098))
- (deps) Update rust crate phonenumber to v0.3.9 ([!2108](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2108))
- (deps) Update rust crate http-request-derive to 0.5.0 ([!2109](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2109))
- (deps) Update rust crate chrono to v0.4.43 ([!2112](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2112))
- (deps) Update rust crate insta to v1.46.1 ([!2115](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2115))
- (deps) Update rust crate rustls-pki-types to v1.13.3 ([!2118](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2118))
- (deps) Update opentalk-types ([!2130](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2130))
- (deps) Lock file maintenance ([!2123](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2123))
- (deps) Update rust crates service-probe and service-probe-client to 0.4 ([!2128](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2128))
- (deps) Update rust crate siphasher to v1.0.2 ([!2146](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2146))
- (deps) Update rust crate moka to v0.12.13 ([!2144](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2144))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag ([!2137](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2137))
- (deps) Update rust crate jsonwebtoken to v10.3.0 ([!2148](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2148))
- (deps) Update rust crate uuid to v1.20.0 ([!2141](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2141))
- (deps) Update rust crate nix to 0.31 ([!2132](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2132))
- (deps) Update rust crate sysinfo to 0.38 ([!2140](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2140))
- (deps) Update rust crate clap to v4.5.55 ([!2150](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2150))
- (deps) Lock file maintenance ([!2143](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2143))
- (deps) Update rust crate clap to v4.5.56 ([!2153](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2153))
- (deps) Update rust crate iana-time-zone to v0.1.65 ([!2151](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2151))
- (deps) Update rust crate aws-sdk-s3 to v1.121.0 ([!2152](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2152))
- (deps) Update rust crate rand_chacha to 0.10 ([!2161](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2161))
- (deps) Update rust crate bytes to v1.11.1 ([!2167](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2167))
- (deps) Update rust crate redis to v1.0.3 ([!2154](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2154))
- (deps) Update rust crate arc-swap to v1.8.1 ([!2163](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2163))
- (deps) Update rust crate insta to v1.46.3 ([!2155](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2155))
- (deps) Update opentalk-types-api-v1 to 0.52.1 ([!2171](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2171))
- (deps) Update opentalk-types-common to 0.42.1 ([!2171](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2171))
- (deps) Update rust crate clap to v4.5.57 ([!2168](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2168))
- (deps) Update rust crate aws-sdk-s3 to v1.122.0 ([!2169](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2169))
- (deps) Lock file maintenance ([!2158](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2158))
- (deps) Update rust crate sysinfo to v0.38.1 ([!2184](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2184))
- (deps) Update livekit ([!2188](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2188))
- (deps) Lock file maintenance ([!2187](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2187))
- (deps) Update rust crate rkyv to v0.8.15 ([!2196](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2196))
- (deps) Update rust crate tempfile to v3.25.0 ([!2195](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2195))
- (deps) Update rust crate reqwest to 0.13 ([!2090](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2090))
- (deps) Update rust crate toml to v0.9.12 ([!2200](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2200))
- (deps) Update rust crate env_logger to v0.11.9 ([!2204](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2204))
- (deps) Update rust crate aws-sdk-s3 to v1.123.0 ([!2209](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2209))
- (deps) Update rust crate clap to v4.5.58 ([!2203](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2203))
- (deps) Update rust crate toml to v1 ([!2208](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2208))
- (deps) Update rust crate config to v0.15.19 ([!2212](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2212))
- (deps) Update rust crate anstream to v1 ([!2207](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2207))
- (deps) Lock file maintenance ([!2223](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2223))
- (deps) Update livekit ([!2227](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2227))
- (deps) Update rust crate clap to v4.5.59 ([!2228](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2228))
- (deps) Update rust crate actix-web to v4.13.0 ([!2238](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2238))
- (deps) Update rust crate redis to v1.0.4 ([!2241](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2241))
- (deps) Update rust crate clap to v4.5.60 ([!2245](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2245))
- (deps) Update rust crate aws-sdk-s3 to v1.124.0 ([!2236](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2236))
- (deps) Update redocly/cli docker tag to v1.34.7 ([!2240](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2240))
- (deps) Update rust crate toml to v1.0.3 ([!2226](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2226))
- (deps) Update rust crate opentalk-types-api-v1 to v0.52.4 ([!2248](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2248))
- (deps) Update rust crate opentalk-types-api-v1 to v0.53.0 ([!2251](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2251))
- (deps) Update rust crate serial_test to v3.4.0 ([!2255](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2255))
- (deps) Update rust crate opentalk-types-api-internal to v0.1.1 ([!2254](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2254))
- (deps) Update rust crate owo-colors to v4.3.0 ([!2257](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2257))
- (deps) Lock file maintenance ([!2258](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2258))
- (deps) Update rust crate rand to 0.10 ([!2185](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2185))
- (deps) Lock file maintenance ([!2273](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2273))
- (deps) Update rust crate moka to v0.12.14 ([!2274](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2274))
- (deps) Update quay.io/buildah/stable docker tag to v1.42.2 ([!2283](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2283))
- (deps) Update rust crate nix to v0.31.2 ([!2277](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2277))
- (deps) Update rust crate chrono to v0.4.44 ([!2276](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2276))
- (deps) Update rust crate pin-project-lite to v0.2.17 ([!2279](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2279))
- (deps) Update redocly/cli docker tag to v1.34.10 ([!2263](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2263))
- (deps) Update rust crate rustls to v0.23.37 ([!2280](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2280))
- (deps) Update rust crate sysinfo to v0.38.3 ([!2284](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2284))
- (deps) Update rust crate url to v2.5.8 ([!2286](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2286))
- (deps) Update rust crate serde_with to v3.17.0 ([!2291](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2291))
- (deps) Update rust crate tempfile to v3.26.0 ([!2293](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2293))
- (deps) Update roomserver types to 0.0.24 ([!2294](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2294))
- (deps) Update rust crate aws-sdk-s3 to v1.125.0 ([!2301](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2301))
- (deps) Update rust crate toml to v1.0.4 ([!2300](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2300))
- (deps) Update rust crate uuid to v1.22.0 ([!2305](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2305))
- (deps) Update aws-lc-sys to 1.16.1 ([!2306](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2306), [#1293](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1293), [#1292](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1292), [#1291](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1291), [#1290](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1290), [#1289](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1289))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag ([!2308](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2308))
- (deps) `opentalk-controller-service`  requires `client` of `opentalk-mail-worker-protocol` ([!2309](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2309))
- (deps) Update rust crate refinery to 0.9 ([!2042](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2042))
- (deps) Update rust crate toml to v1.0.6 ([!2313](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2313))
- (deps) Update quay.io/buildah/stable docker tag to v1.43.0 ([!2314](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2314))
- (deps) Update rust crate redis to v1.0.5 ([!2316](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2316))

### ⚙ Miscellaneous

- Fix spelling ([!1784](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1784))
- Ensure `NewAssetFileName` can be used in tracing fields ([!1784](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1784))
- Update roomserver client to use opentalk-service-auth ([!1986](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1986))
- (cache) Test exisiting functionality of the local cache ([!2122](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2122))
- (cache) Test hashing wrapper ([!2142](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2142))
- (ci) Set release creation ci template ref to `v1` ([!2216](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2216))
- Cleanup unused mod ([!2296](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2296))
- (container) Remove debian bookworm image ([!2317](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2317), [#1296](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1296))

### Ci

- Publish docs locally on gitlab-pages ([!2019](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2019))

### Test

- (inventory) Add test for `UpdateEvent::is_time_independent` ([!2173](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2173))

## [0.32.13] - 2026-06-18

[0.32.13]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.12...v0.32.13

### 🚀 New features

- (keycloak) Adapt to optional name for keycloak users ([!2594](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2594), [#1431](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1431))

### 🐛 Bug fixes

- (auth) Active user with empty name must be able to make requests ([!2594](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2594), [#1431](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1431))

### 📦 Dependencies

- (deps) Update rust crate `tokio-postgres` to 0.7.18 ([!2592](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2592), [#1427](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1427))

### ⚙ Miscellaneous

- Resolve new clippy linting rules ([!2604](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2604))
- Update pre-commit config ([!2604](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2604))

### Ci

- Use container build template ([!2604](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2604), [#1379](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1379))
- (just) Don't consider tags from other branches ([!2604](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2604))

## [0.32.12] - 2026-06-03

[0.32.12]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.11...v0.32.12

### 🐛 Bug fixes

- (livekit-proxy) Allow popout steams ([!2571](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2571), [#1420](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1420))

### Ci

- Build dev images for release branches ([!2571](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2571), [#1379](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1379))

## [0.32.11] - 2026-04-08

[0.32.11]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.10...v0.32.11

### 📦 Dependencies

- (deps) Update build toolchain to 1.94 for 0.32 release series ([!2417](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2417), [#1349](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1349))

## [0.32.10] - 2026-04-07

[0.32.10]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.9...v0.32.10

### 🐛 Bug fixes

- (api) Return 403 if room event belongs to a disabled user ([!2391](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2391))

### 📦 Dependencies

- (deps) Lockfile maintenance ([!2416](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2416))

### ⚙ Miscellaneous

- (ci) Include check-changelog pipeline instead of defining it in this project ([!2416](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2416))

## [0.32.9] - 2026-03-19

[0.32.9]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.8...v0.32.9

### 📦 Dependencies

- (deps) Explicitly opt in to `aws-lc-rs` crypto backend ([!2366](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2366), [#1320](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1320))

## [0.32.8] - 2026-03-17

[0.32.8]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.7...v0.32.8

### 🐛 Bug fixes

- (e2ee) Verify invites processing ([!2310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2310), [#1283](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1283))
- (e2ee) Remove guests allowed feature ([!2310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2310), [#1283](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1283))
- (api) Block the creation of invite codes for e2ee events ([!2310](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2310), [#1283](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1283))
- (docs) Document that the controller must be restarted after `fix-acl` ([!2324](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2324), [#1300](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1300))
- (assets) Handle headers with non-ASCII characters ([!2340](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2340), [#1310](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1310))
- (livekit-proxy) Consider whisper groups when checking participant's room state ([!2343](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2343), [#1306](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1306))
- (livekit-proxy) Add missing GET rtc/validate & rtc/v1/validate endpoint ([!2353](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2353), [#1312](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1312))

### Ci

- (just) Update changelog tool to opentalk git-cliff ([!2334](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2334))

## [0.32.7] - 2026-03-02

[0.32.7]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.6...v0.32.7

### 🚀 New features

- (ci) Switch to buildah ([!2282](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2282))

### 🐛 Bug fixes

- (core) Backport proxy livekit connection to workaround livekit auth limitations ([!2282](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2282), [#1270](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1270))

### 📦 Dependencies

- (deps) Lockfile maintenance for 0.32 release series ([!2281](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2281))

## [0.32.6] - 2026-02-25

[0.32.6]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.5...v0.32.6

### 🚀 New features

- (assets) Add endpoint for short lived download URLs ([!2268](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2268))

### 🐛 Bug fixes

- (ci) Set correct ref for release creation ci template repository for 0.32 release branch ([!2211](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2211))

### ⚙ Miscellaneous

- (ci) Fix ref for release creation ci template ([!2268](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2268))

## [0.32.5] - 2026-02-11

[0.32.5]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.4...v0.32.5

### 🚀 New features

- (ci) Add release mr creation job ([!2205](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2205))

### 🐛 Bug fixes

- (training-participation-report) Remove unreachable! statements ([!2191](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2191))

## [0.32.4] - 2026-02-06

[0.32.4]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.3...v0.32.4

### 🚀 New features

- Conditionally set x-forwarded-host header in oidc introspect and userinfo requests ([!2179](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2179))

### 📦 Dependencies

- (deps) Update dependencies in order to address RUSTSEC advisories ([!2180](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2180))

## [0.32.3] - 2026-01-23

[0.32.3]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.2...v0.32.3

### 🐛 Bug fixes

- (api-v1) Overwrite `shared_folder` with patched state ([!2114](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2114))
- (db) Properly handle users which have no language set ([!2126](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2126))

### 📦 Dependencies

- (deps) Update dependency linguify to v0.5.0 ([!2114](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2114))
- (deps) Document accepted RUSTSEC advisories ([!2126](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2126))
- (deps) Update aws-smithy-runtime from yanked 1.9.6 to latest semver-compatible version ([!2126](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2126))

### ⚙ Miscellaneous

- (ci) Update ci runs to rust 1.92 container ([!2126](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2126))

## [0.32.2] - 2025-12-23

[0.32.2]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.32.1...v0.32.2

### 🚀 New features

### 🐛 Bug fixes

- (api) Correctly communicate 404 for missing database entries instead of 500 ([!2063](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2063), [#1206](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1206))

### 📦 Dependencies

- (deps) Lockfile maintenance ([!2079](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2079), [#1209](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1209))

### Ci

- (ci) Fix rules for container tag creation ([!2078](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2078))
- (ci) Load images only if necessary ([!1996](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1996))
- (ci) Push images to new registry ([!1980](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1980), [#1191](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1191))
- (ci) Include commit evidence job ([!2073](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2073), [#1208](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1208))

## [0.32.1] - 2025-11-28

[0.32.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/prepare-docs-for-mkdocs-material-v0.31.3...v0.32.1

### 🚀 New features

- (docs) Prepare documentation for mkdocs-material ([!1993](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1993), [#1145))

### 📦 Dependencies

- (deps) Update opentalk-types-api-v1 to 0.46.1, allowing invitees_max parameter to be 0 ([#1190](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1190))

## [0.32.0] - 2025-11-13

[0.32.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.31.3...v0.32.0

### 🚀 New features

- Add report creation date to attendance reports ([!1738](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1738))
- (rabbitmq) Add time-to-live to outgoing RabbitMQ messages ([!1740](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1740), [#1115](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1115))
- Include moderator to training participation report ([!1751](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1751))
- Include linguify typst package in container image ([!1899](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1899))
- Add from scratch build ([!1859](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1859))
- Add auditable builds ([!1859](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1859))
- Add from scratch to matrix build ([!1859](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1859))
- Use clux/muslrust image ([!1859](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1859))
- (ci) Make MR container build optional ([!1941](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1941))
- Send `waiting_room_disabled` event to waiting participants ([!1945](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1945), [#1127](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1127))
- Accept all waiting participants when disabling the waiting room ([!1945](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1945), [#1127](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1127))
- (l10n) Generate pdf reports in the language of the room owner ([!1594](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1594), [#948](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/948))
- Propagate `not_found` error ([!1953](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1953))
- (settings) Introduce CORS configuration ([!1870](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1870), [#1147](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1147))

### 🐛 Bug fixes

- (modules) Remove unused integrations signaling module ([!1753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1753))
- Create appdata of type `Caches` instead of `Arc<Caches>` ([!1778](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1778))
- (livekit) Exclude moderators from screenshare restrictions ([!1783](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1783))
- (meeting-report) Calculate correct `ends_at` for recurring meetings ([!1806](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1806))
- (legal_vote) Always enable module, even if first participant is a guest ([!1819](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1819), [#1130](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1130))
- Recorder timeout message when stopped before timeout hit ([!1741](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1741))
- Don't return an internal error but a client error when storage quota is exceeded ([!1839](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1839))
- Determine correct `invitees_to_skip` per page ([!1868](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1868))
- (roomserver) Handle failed token request when user is banned ([!1875](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1875))
- Don't retry to acquire participant id in participant_runner_lock tests ([!1900](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1900))
- (ci) Run all tests on nightly ([!1902](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1902))
- (ci) Code coverage is not displayed in MRs ([!1933](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1933))
- (ci) Improve test timing for expiring_data module ([!1942](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1942))
- (storage) Make long-lasting uploads such as recordings robust against database connection drops ([!1946](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1946), [#1163](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1163))
- (training) Handle fixed zero "within" timespan without crashing ([!1949](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1949), [#1166](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1166))
- Return a 404 HTTP status when an entity was not found ([!1953](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1953), [#1169](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1169))
- (roomserver) Override module config settings with values from the database ([!1952](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1952), [#1162](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1162))

### 📚 Documentation

- (db) Remove `is_recurring` from `events` ([!1874](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1874))

### 🔨 Refactor

- (inventory) use specific inventory types instead of database types in inventory API ([!1708](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1708))
- Remove unneeded Arcs in global memory state ([!1735](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1735))
- Remove duplicate function definition for services ([!1734](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1734))
- (modules) Module initialization and organization ([!1753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1753))
- (modules) Move echo module into separate crate ([!1753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1753))
- (modules) Move breakout module into separate crate ([!1753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1753))
- (modules) Move moderation module into separate crate ([!1753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1753))
- (modules) Remove ControllerModules type, use Modules directly ([!1753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1753))
- (modules) Make modules registration sync ([!1753](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1753))
- Add inventory facade for kustos ([!1760](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1760))
- (service) Move update of token cache into service ([!1771](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1771))
- Introduce pagination NewTypes ([!1813](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1813))
- (modules) Prevent instantiation of e2ee-incompatible signaling modules ([!1820](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1820), [#1131](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1131))
- Use new datatypes that were moved to opentalk-types-api-v1 ([!1855](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1855))
- (db) Remove `is_recurring` from `events` ([!1874](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1874))
- Use path for `MODULE_ID` to remove ambiguity ([!1945](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1945))
- Don't include `enable` in the name as this might be confusing ([!1945](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1945))

### 📦 Dependencies

- (deps) Update rust crate snafu to v0.8.7 ([!1720](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1720))
- (deps) Update rust crate cargo_metadata to 0.22 ([!1710](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1710))
- (deps) Update rust crate url to v2.5.6 ([!1724](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1724))
- (deps) Update rust crate serde_json to v1.0.143 ([!1711](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1711))
- (deps) Lock file maintenance ([!1729](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1729))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.9 ([!1718](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1718))
- (deps) Update rust crate actix-http to v3.11.1 ([!1731](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1731))
- (deps) Update rust crate clap to v4.5.46 ([!1732](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1732))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.10 ([!1737](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1737))
- (deps) Update rust crate snafu to v0.8.8 ([!1739](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1739))
- (deps) Lock file maintenance ([!1747](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1747))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.11 ([!1746](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1746))
- (deps) Update rust crate uuid to v1.18.1 ([!1748](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1748))
- (deps) Update rust crate clap to v4.5.47 ([!1749](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1749))
- (deps) Update rust crate log to v0.4.28 ([!1754](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1754))
- (deps) Update rust crate snafu to v0.8.9 ([!1752](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1752))
- (deps) Update rust crate insta to v1.43.2 ([!1756](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1756))
- (deps) Update pre-commit hook fsfe/reuse-tool to v5.1.0 ([!1755](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1755))
- (deps) Update pre-commit hook fsfe/reuse-tool to v5.1.1 ([!1758](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1758))
- (deps) Lock file maintenance ([!1759](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1759))
- (deps) Update rust crate tempfile to v3.22.0 ([!1764](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1764))
- (deps) Update rust crate chrono to v0.4.42 ([!1761](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1761))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.12 ([!1766](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1766))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.5 ([!1770](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1770))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.5 ([!1785](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1785))
- (deps) Lock file maintenance ([!1776](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1776))
- (deps) Update rust crate opentalk-etherpad-client to 0.3.0 ([!1765](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1765))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.90.0 ([!1788](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1788))
- (deps) Update rust crate opentalk-roomserver-types to 0.0.7 ([!1781](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1781))
- (deps) Update rust crate config to v0.15.17 ([!1792](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1792))
- (deps) Update rust crate serde to v1.0.227 ([!1794](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1794))
- (deps) Lock file maintenance ([!1801](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1801))
- (deps) Update opentalk-roomserver to 0.0.9 ([!1805](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1805))
- (deps) Update rust crate opentalk-etherpad-client to 0.4.0 ([!1808](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1808))
- (deps) Use opentalk-report-generation from crates.io ([!1810](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1810))
- (deps) Update rust crate livekit ([!1802](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1802))
- (deps) Update opentalk-types ([!1813](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1813))
- (deps) Update opentalk-roomserver crates ([!1813](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1813))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.23.0 ([!1821](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1821))
- (deps) Update rust crate cargo_metadata to 0.23 ([!1804](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1804))
- (deps) Update rust crate pdf-extract to 0.10 ([!1826](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1826))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6 ([!1829](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1829))
- (deps) Update rust crate tokio-cron-scheduler to 0.15 ([!1799](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1799))
- (deps) Lock file maintenance ([!1834](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1834))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.6 ([!1843](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1843))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.7 ([!1848](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1848))
- (deps) Lock file maintenance ([!1852](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1852))
- (deps) Update postgres docker tag to v18 ([!1795](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1795))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.8 ([!1853](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1853))
- (deps) Update rust crate opentalk-types-api-v1 to 0.43.0 ([!1854](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1854))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.9 ([!1857](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1857))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.0 ([!1867](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1867))
- (deps) Update rust crate rustls to v0.23.34 ([!1860](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1860))
- (deps) Update rust crate clap to v4.5.50 ([!1856](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1856))
- (deps) Lock file maintenance ([!1872](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1872))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.1 ([!1871](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1871))
- (deps) Update opentalk-roomserver to 0.0.11 ([!1875](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1875))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6.2.0 ([!1877](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1877))
- (deps) Update rust crate opentalk-version to 0.3.0 ([!1878](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1878))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.2 ([!1890](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1890))
- (deps) Update rust crate clap to v4.5.51 ([!1896](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1896))
- (deps) Lock file maintenance ([!1904](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1904))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.4 ([!1898](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1898))
- (deps) Update rust crate etcd-client to 0.17 ([!1910](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1910))
- (deps) Update rust crate rustls to v0.23.35 ([!1920](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1920))
- (deps) Update opentalk-report-generation to 0.2.0 ([!1935](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1935))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.91.0 ([!1901](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1901))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.91.0 ([!1937](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1937))
- (deps) Update rust crate aws-sdk-s3 to v1.111.0 ([!1934](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1934))
- (deps) Update opentalk-types-common to 0.39.0 ([!1938](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1938))
- (deps) Update rust crate aws-sdk-s3 to v1.112.0 ([!1939](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1939))
- (deps) Update opentalk-roomserver to 0.0.14 ([!1944](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1944))

### ⚙ Miscellaneous

- (docs) Make casing and links more consistent ([!1723](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1723))
- (ci) Update opentalk-ci-doc-updater image to 0.2.0 ([!1725](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1725))
- Update default ci and container image to Debian Trixie ([!1725](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1725), [#1107](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1107))
- Switch to internal kaniko image ([!1789](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1789), [#1120](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1120))
- (renovate) Group roomserver updates ([!1781](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1781))
- (renovate) Group livekit updates ([!1802](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1802))
- (renovate) Ensure crate names are matched from start ([!1818](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1818))
- (renovate) Update config to new `matchPackageNames` ([!1889](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1889))
- (renovate) Types updates require a roomserver update ([!1889](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1889))
- (ci) Remove obsolete mod.rs file check ([!1940](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1940))

### Ci

- Use cargo nextest for tests ([!1895](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1895))
- Use junit for code coverage ([!1895](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1895))
- Add doctests ([!1895](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1895))

## [0.31.4] - 2026-03-03

[0.31.4]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.31.3...v0.31.4

### 🚀 New features

- (docs) Prepare documentation for mkdocs-material ([!1983](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1983), [#1145](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1145))
- (ci) Switch to buildah ([!2285](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2285))
- (ci) Introduce release creation and commit evidence collection jobs in 0.31 release series ([!2288](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2288))

### 🐛 Bug fixes

- (core) Proxy livekit connection to workaround livekit auth limitations ([!2285](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2285), [#1267](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1267))
- (ci) Adjust justfile for automatic release mr creation on the 0.31 branch ([!2289](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2289))

### 📦 Dependencies

- (deps) Lockfile maintenance, build environment and deny.toml update for controller 0.31 ([!2275](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/2275))

## [0.31.3] - 2025-11-11

[0.31.3]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.31.2...v0.31.3

### 🐛 Bug fixes

- (storage) Make long-lasting uploads such as recordings robust against database connection drops ([!1947](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1947), [#1164](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1164))

## [0.31.2] - 2025-10-22

[0.31.2]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.31.1...v0.31.2

### ⚙ Miscellaneous

- (ci) Update opentalk-ci-doc-updater image to 0.2.0 ([!1865](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1865))
- Update default ci and container image to Debian Trixie ([!1865](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1865), [#1107](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1107))

## [0.31.1] - 2025-10-14

[0.31.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.31.0...v0.31.1

### 🐛 Bug fixes

- (legal_vote) Always enable module, even if first participant is a guest ([!1841](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1841), [#1130](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1130))

### 📦 Dependencies

- (deps) Fixes for updated rust version ([!1841](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1841))
- (deps) Update `tracing-subscriber` to v0.3.20 ([!1841](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1841))

## [0.31.0] - 2025-08-21

[0.31.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.30.3...v0.31.0

### 🚀 New features

- (migrations) Add migration to update ends_at for recurring events ([!1609](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1609), [bugs#113](https://git.opentalk.dev/opentalk/bugs/-/issues/113))
- (metrics) Show warning on metrics denial ([!1616](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1616), [#1051](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1051))
- (metrics) Add room life time ([!1620](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1620))
- (metrics) Add participant meeting time ([!1620](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1620))
- (metrics) Add participants per room ([!1620](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1620))
- (settings) Add `Frontend` to config ([!1627](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1627), [#1052](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1052))
- (settings) Add `OperatorInformation` to config ([!1627](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1627), [#1052](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1052))
- (settings) Update `example/controller.toml` ([!1627](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1627), [#1052](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1052))
- (cli) Print license information ([!1573](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1573))
- (chat) Chunk message history and add server side history search ([!1637](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1637), [#1071](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1071), [bugs#137](https://git.opentalk.dev/opentalk/bugs/-/issues/137))
- (timezone) Determine fallback timezone from configuration and/or environment variables ([!1671](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1671), [#1090](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1090))
- (control) Kick participants when they exceed the websocket rate limit ([!1655](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1655), [#1072](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1072))
- (auth) Implement guests_allowed module feature ([!1704](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1704), [#1082](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1082))
- (api) Add GET /events/instances endpoint ([!1632](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1632), [#1061](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1061))
- (roomserver) Add experimental start endpoints for the roomserver signaling ([!1621](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1621), [#994](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/994))
- (roomserver) Set experimental roomserver modules via config ([!1652](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1652), [#1083](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1083))

### 🐛 Bug fixes

- (config) Install configuration to /etc/opentalk/controller.toml in container ([!1612](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1612), [#1059](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1059))
- (migrations) Check for invalid quotas, modules and features for tariffs ([!1613](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1613), [#772](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/772))
- (metrics) Add content type for the metrics endpoint ([!1617](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1617))
- (api) Make paginated query work in async trait functions ([!1619](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1619))
- (api) Don't expose expired invite links in meeting details ([!1645](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1645), [#1074](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1074))
- (api) Fix `NoBreakoutRooms` error on start_invited endpoint ([!1649](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1649))
- (oidc) Compare e-mail address in lowercase when updating user from OIDC information ([!1656](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1656), [#1084](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1084))
- (auth) User creation race condition resulting in unique constraint violation ([!1662](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1662), [#1086](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1086))
- (db) Properly compare expiration date for valid room invites ([!1665](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1665))
- (db) User creation race condition which is still present in database transaction ([!1670](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1670), [#1094](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1094))
- (chat) Redis args for last seen timestamps ([!1637](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1637), [#1071](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1071))
- (oidc) Handle optional `exp` field ([!1667](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1667), [#1092](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1092))
- (recording) Improve error message when recorder is unavailable ([!1615](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1615))

### 📚 Documentation

- (openapi) Align documentation with behavior ([!1697](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1697), [#1097](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1097))
- (config) Improve consistency and documentation of user search configuration ([!1610](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1610), [#1058](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1058))
- (config) Add `operator_information` section to admin docs ([!1715](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1715), [#1109](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1109))
- (config) Add `frontend` section to admin docs ([!1716](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1716), [#1110](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1110))
- (config) Use default values as examples in example.toml ([!1642](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1642), [#869](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/869))
- Change index of admin docs to clarify requirement of redis and rabbitmq ([!1719](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1719), [#1098](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1098))
- Make links to Keycloak documentation point to latest ([!1717](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1717), [#1095](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1095))
- Fix some rough edges in the documentation ([!1623](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1623))

### 🔨 Refactor

- (metric) Restructure metric initialization ([!1620](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1620))
- (database) Return vec instead of single item from module resource patch function ([!1630](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1630), [#1070](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1070))
- (database) Create opentalk-inventory database abstraction layer ([!1631](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1631), [#1068](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1068))
- (recording) Move module into a separate crate ([!1638](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1638))
- (inventory) Use inventory interface instead of direct database access ([!1614](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1614), [#1069](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1069))
- (chat) Move `RoomPrivateChatHistory` to a separate module ([!1637](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1637), [#1071](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1071))
- (tariff) Enforce tariff features consistently in all locations ([!1700](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1700), [#1103](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1103))
- (code) Remove unused api v1 streaming services file ([!1657](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1657))

### 📦 Dependencies

- Update all dependencies to the latest version possible ([!1679](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1679))
- Update alpine docker tag to v3.22 ([!1611](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1611))
- Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.89.0 ([!1634](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1634), [!1690](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1690))
- Update opentalk-types ([!1604](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1604))
- Update pre-commit hook daveshanley/vacuum to v0.17.8 ([!1673](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1673), [!1668](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1668), [!1658](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1658), [!1651](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1651))
- Update pre-commit hook embarkstudios/cargo-deny to v0.18.4 ([!1705](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1705))
- Update pre-commit hook pre-commit/pre-commit-hooks to v6 ([!1691](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1691))
- Update redocly/cli docker tag to v1.34.5 ([!1660](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1660), [!1635](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1635))
- Update rust crate async-trait to v0.1.89 ([!1706](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1706))
- Update rust crate aws-sdk-s3 to v1.102.0 ([!1702](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1702))
- Update rust crate bincode to v2 ([!1534](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1534))
- Update rust crate cargo_metadata to 0.21 ([!1647](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1647), [!1606](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1606))
- Update rust crate clap to v4.5.45 ([!1699](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1699), [!1696](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1696))
- Update rust crate config to v0.15.14 ([!1701](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1701))
- Update rust crate md5 to 0.8 ([!1633](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1633))
- Update rust crate sysinfo to 0.37 ([!1693](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1693), [!1648](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1648))
- Update rust crate tabled to 0.20 ([!1618](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1618))
- Update rust crate tempfile to v3.21.0 ([!1714](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1714))
- Update rust crate toml to 0.9 ([!1646](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1646))
- Update rust crate uuid to v1.18.0 ([!1695](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1695))
- Downgrade lapin and lapin-pool for now while the new versions are broken ([!1688](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1688), [#1101](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1101))
- Lock file maintenance ([!1709](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1709), [!1676](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1676), [!1694](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1694))

### ⚙ Miscellaneous

- Update pre-commit hooks ([!1644](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1644))
- Update to rust edition 2024 ([!1643](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1643))
- Add ignoreDeps renovate rule for actix-web-actors ([!1674](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1674))
- Fix clippy lints ([!1676](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1676), [!1687](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1687))
- (ci) Remove rabbitmq service from check:docs-generated-parts job ([!1628](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1628))

## [0.30.6] - 2025-11-11

[0.30.6]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.30.5...v0.30.6

### 🐛 Bug fixes

- (storage) Make long-lasting uploads such as recordings robust against database connection drops ([!1948](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1948), [#1165](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1165))

## [0.30.5] - 2025-10-22

[0.30.5]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.30.4...v0.30.5

### ⚙ Miscellaneous

- (ci) Update opentalk-ci-doc-updater image to 0.2.0
- Update default ci and container image to Debian Trixie ([#1107](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1107))

## [0.30.4] - 2025-10-13

[0.30.4]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.30.3...v0.30.4

### 🚀 New features

- (rabbitmq) Add time-to-live to outgoing RabbitMQ messages ([!1832](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1832), [#1115](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1115))

### 📦 Dependencies

- (deps) Update `slap`, `tracing-subscriber` ([!1832](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1832))

## [0.30.3] - 2025-08-07

[0.30.3]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.30.2...v0.30.3

### 🐛 Bug fixes

- (db) User creation race condition which is still present in database transaction ([#1094](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1094))

## [0.30.2] - 2025-07-25

[0.30.2]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.30.1...v0.30.2

### 🐛 Bug fixes

- User creation race condition resulting in unique constraint violation ([!1661](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1661), [#1086](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1086))
- (oidc) Compare e-mail address in lowercase when updating user from OIDC information ([!1664](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1664), [#1084](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1084))
- Don't expose expired invite links in meeting details ([!1664](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1664), [#1074](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1074))

### 📦 Dependencies

- (deps) Update minor dependency versions ([!1664](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1664))

## [0.30.1] - 2025-06-17

[0.30.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.30.0...v0.30.1

### 🐛 Bug fixes

- Install configuration to /etc/opentalk/controller.toml in container ([!1612](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1612), [#1059](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1059)
- (config) Improve consistency and documentation of user search configuration ([!1610](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1610), [#1058](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1058)

## [0.30.0] - 2025-05-29

[0.30.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.29.5...v0.30.0

### 🚀 New features

- Implement e2ee signaling ([!1443](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1443))
- (training_participation_report) Add configuration to api ([!1479](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1479), [#972](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/972))
- (training_participation_report) Communicate parameters to frontend on join ([!1479](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1479))
- (training_participation_report) Start training participation report procedure automatically ([!1479](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1479))
- Add `automod` to `controller` ([!1523](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1523))
- Remove obsolete opentalk-api-client crate ([!1550](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1550))
- (OIDC) Get user's timezone from JWT ([!1552](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1552))
- (api) Add .well-known/opentalk/api information endpoint ([!1554](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1554), [#1001](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1001))
- Implement Eq and PartialEq for the Settings ([!1561](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1561))
- Add `legal-vote` to `controller` ([!1562](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1562))
- Read Accept-Language header or JWT locale for detecting default account language ([!1570](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1570), [#914](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/914))
- (settings) Load configuration from a list of commonly used locations ([!1581](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1581), [#992](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/992))
- Use timezone from user for report generation ([!1585](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1585), [#1008](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1008))
- Make max_storage more human readable ([!1593](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1593), [#767](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/767))
- Add storage_upgradable module feature ([!1592](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1592), [#1043](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1043))
- (keycloak-account-sync) Add option to dump failed responses ([!1588](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1588))
- Add kicked and banned leave reasons ([!1599](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1599))
- Catch invalid quota types properly ([!1602](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1602))
- (settings) Remove turn and stun settings ([!1603](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1603), [#910](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/910))
- Remove k3k backwards compatibility helpers ([!1607](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1607), [#1055](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1055))

### 🐛 Bug fixes

- Address cargo-deny remarks ([!1480](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1480))
- (timer) Cleanup on room destroy ([!1483](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1483), [#975](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/975))
- Exclude moderators from microphone restrictions ([!1485](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1485), [#978](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/978))
- (event) Handle missing participation report parameter set correctly ([!1491](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1491), [#979](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/979))
- (meeting_report) Include users who already left the meeting ([!1492](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1492), [#981](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/981))
- (training-participation-report) Don't fail when updating the database entry ([!1493](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1493), [#982](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/982))
- (training-participation-report) Parameter set not stored for unscheduled events ([!1507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1507), [#987](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/987))
- Log internal error with error level ([!1529](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1529))
- (training-participation-report) Autostart when non-trainer joins first ([!1545](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1545), [#998](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/998))
- (db) Consistently exclude from queries events owned by disabled users ([!1558](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1558), [#1007](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1007))
- (settings) Properly load turn.lifetime field ([!1577](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1577), [#1039](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1039))
- Hide call-in info in emails for encrypted rooms ([!1579](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1579))
- The call in requires the country code to be included in the phone number ([!1578](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1578))
- (legal-vote) Skip final results in report generation if none are present ([!1562](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1562))
- Remove shared folder from database when deleted while patching event ([!1597](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1597))
- (livekit) Handle room destruction gracefully ([!1598](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1598))
- (metrics) Properly track metrics for created and destroyed rooms ([!1601](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1601), [#976](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/976))

### ⚡ Performance

- (db) Add index on casbin_rule(ptype,v1) ([!1551](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1551))

### 📚 Documentation

- Sort listing of configurations alphabetically ([!1532](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1532))
- Add OIDC and User search to config sections ([!1532](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1532))
- Add deprecation waring to `report` section ([!1532](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1532))
- Add `automod` to `modules.md` ([!1523](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1523))
- Add `legal_vote` to `module.md` ([!1562](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1562))

### 🔨 Refactor

- (opentalk-api-client) Remove client related code, only keep request types ([!1535](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1535))
- (settings) Introduce SettingsProvider ([!1561](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1561), [#1012](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1012))
- Redesign OpenID Connect integration ([!1548](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1548))
- Move SettingsError into a separate module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move SettingsLoading into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Extensions into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move WarningSource into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move MonitoringSettings into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move OidcAndUserSearchConfiguration into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move OidcConfiguration into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move FrontendOidcConfiguration into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move ControllerOidcConfiguration into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move UserSearchConfiguration into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Database into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Keycloak into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Oidc into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move OidcFrontend into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move OidcController into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move UserSearch into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move UserSearchBackend into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move UsersFindBehavior into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Http into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move HttpTls into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Logging into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Turn into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move TurnServer into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Stun into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move RedisConfig into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move RabbitMqConfig into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Authz into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Etcd into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Etherpad into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Spacedeck into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move SubroomAudio into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Reports into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move ReportsTemplate into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move SharedFolder into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Avatar into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move CallIn into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Defaults into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Endpoints into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move MinIO into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Metrics into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move TenantAssignment into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Tenants into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move TariffAssignment into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move TariffStatusMapping into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move Tariffs into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Move LiveKitSettings into settings_file module ([!1563](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1563))
- (settings) Rename Settings alias to SettingsRaw ([!1564](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1564))
- (settings) Introduce a new Settings struct which will hold the runtime settings ([!1564](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1564))
- (settings) Access raw settings through field in runtime settings only ([!1564](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1564))
- (settings) Move loading and deprecation checking into SettingsProvider ([!1564](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1564))
- (settings) Add oidc and user search settings to runtime configuration ([!1564](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1564))
- (settings) Move http configuration to runtime settings ([!1568](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1568), [#1014](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1014))
- (settings) Move database configuration to runtime settings ([!1571](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1571), [#1036](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1036))
- (settings) Move turn configuration to runtime settings ([!1571](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1571), [#1015](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1015))
- (settings) Move stun configuration to runtime settings ([!1571](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1571), [#1016](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1016))
- (settings) Move redis configuration to runtime settings ([!1571](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1571), [#1017](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1017))
- (settings) Move rabbitmq and authz configuration to runtime settings ([!1574](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1574), [#1018](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1018))
- (legal-vote) Remove `ErrorKind::BadRequest` ([!1562](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1562))
- (settings) Move logging configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1019](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1019))
- (settings) Move avatar configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1021](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1021))
- (settings) Move metrics configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1022](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1022))
- (settings) Move etcd configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1023](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1023))
- (settings) Move etherpad configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1024](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1024))
- (settings) Move spacedeck configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1025](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1025))
- (settings) Move subroom audio configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1026](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1026))
- (settings) Mark report configuration `pub(crate)` in settings file ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1027](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1027))
- (settings) Move shared folder configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1028](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1028))
- (settings) Move endpoints configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1030](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1030))
- (settings) Move minio configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1031](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1031))
- (settings) Move monitoring configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1032](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1032))
- (settings) Move call-in configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1029](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1029))
- (settings) Move tenants configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1033](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1033))
- (settings) Move tariffs configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1034](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1034))
- (settings) Move defaults configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1041](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1041))
- (settings) Move livekit configuration to runtime settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1035](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1035))
- (settings) Remove SettingsRaw from runtime Settings ([!1575](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1575), [#1040](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1040), [#941](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/941))
- Rename `opentalk-community-signaling-modules` to `opentalk-signaling-modules` ([!1582](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1582))
- Remove `controller-enterprise` from GitLab CI ([!1582](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1582))
- (settings) Use `Path` and `PathBuf` for loading the settings ([!1581](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1581))

### 📦 Dependencies

- (deps) Update ring to 0.17.13 ([!1478](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1478))
- (deps) Update rust crate rand to 0.9 ([!1418](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1418))
- (deps) Update redocly/cli docker tag to v1.33.1 ([!1482](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1482))
- (deps) Update redocly/cli docker tag to v1.34.0 ([!1487](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1487))
- (deps) Update crate zip to 2.4.2 ([!1490](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1490))
- (deps) Update redocly/cli docker tag to v1.34.1 ([!1530](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1530))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.86.0 ([!1537](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1537))
- (deps) Update rust crate http-request-derive to 0.4.0 ([!1535](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1535))
- (deps) Update crate opentelemetry to 0.29.1 (and related crates) ([!1549](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1549))
- (deps) Update crate crossbeam-channel to 0.5.15 ([!1549](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1549))
- (deps) Update crate tokio to 1.44.2 ([!1549](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1549))
- (deps) Update redocly/cli docker tag to v1.34.2 ([!1544](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1544))
- (deps) Update rust crates prometheus and opentelemetry-prometheus ([!1553](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1553))
- (deps) Update rust crate sysinfo to 0.34 ([!1528](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1528))
- (deps) Update rust crate pdf-extract to 0.9 ([!1538](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1538))
- (deps) Update rabbitmq docker tag to v4.1 ([!1560](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1560))
- (deps) Update redocly/cli docker tag to v1.34.3 ([!1566](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1566))
- (deps) Update rust crate rrule to 0.14 ([!1565](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1565))
- (deps) Update rust crate etcd-client to 0.15 ([!1556](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1556))
- (deps) Update rust crate tabled to 0.19 ([!1576](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1576))
- (deps) Update rust crate nix to 0.30 ([!1580](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1580))
- (deps) Update redis docker tag to v8 ([!1584](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1584))
- (deps) Update rust crate sysinfo to 0.35 ([!1583](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1583))
- (deps) Update opentalk-types, redis and redis-args ([!1589](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1589))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.87.0 ([!1596](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1596))
- (deps) Update opentalk-types crates ([!1600](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1600))
- (deps) Update rust crate tokio-cron-scheduler to 0.14 ([!1586](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1586))

### ⚙ Miscellaneous

- Unregister e2ee module due to frontend incompatibility ([!1496](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1496))
- (justfile) Add commit release script ([!1322](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1322))
- Add pre-commit config ([!1500](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1500))
- Add pre-commit config ([!1511](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1511))
- Remove deprecated report section from example config ([!1529](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1529))
- Update opentalk-types to 0.33.0 ([!1524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1524))
- Fix openapi-doc block for PATCH /users/me ([!1533](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1533))
- Revise casing of Keycloak ([!1572](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1572))
- (settings) Move `extra/example.toml` to `example/controller.toml` in repository ([!1581](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1581))
- Add cargo-machete to pre-commit config ([!1605](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1605))

### Ci

- Restrict mr container tag lengh to 63 characters ([!1475](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1475), [#971](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/971))
- Configure renovate merge request reviewers ([!1499](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1499))
- Introduce renovate group for opentalk-types ([!1524](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1524))
- Ingore advisory RUSTSEC-2025-0021 ([!1535](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1535))
- Add trivyignore files ([!1547](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1547))
- Add container scanning ([!1543](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1543))
- Hide inclusion graph in cargo-deny output ([!1549](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1549))
- Correct handling of trivyignore files ([!1555](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1555))

### Test

- Use ChaCha12Rng instead of StdRng for reproducibility ([!1418](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1418))

## [0.29.6] - 2025-06-23

[0.29.6]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.29.5...v0.29.6

### 🚀 New features

- (keycloak-account-sync) Add option to dump failed responses ([!1588](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1588), [#841](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/841))

### 🐛 Bug fixes

- Remove shared folder from database when deleted while patching event ([!1597](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1597))

### 📦 Dependencies

- (deps) Lockfile maintenance ([!1629](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1629))

## [0.29.5] - 2025-05-07

[0.29.5]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.29.4...v0.29.5

### 🚀 New features

- (api) Add .well-known/opentalk/api information endpoint ([!1554](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1554), [#1001](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1001))

### 🐛 Bug fixes

- Hide call-in info in emails for encrypted rooms ([!1579](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1579), [#1042](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1042))

## [0.29.4] - 2025-04-15

[0.29.4]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.29.3...v0.29.4

### 🐛 Bug fixes

- (db) Consistently exclude from queries events owned by disabled users ([#1007](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/1007))

## [0.29.3] - 2025-04-09

[0.29.3]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.29.2...v0.29.3

### 🐛 Bug fixes

- (training-participation-report) Autostart when non-trainer joins first ([#998](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/998))

### 📦 Dependencies

- (deps) Update rust crate tokio to v1.44.2 ([!1546](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1546))
- (deps) Update rust crate crossbeam-channel to v0.5.14 ([!1546](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1546))

## [0.29.2] - 2025-04-07

[0.29.2]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.29.1...v0.29.2

### 🐛 Bug fixes

- (training-participation-report) Don't fail when updating the database entry ([#982](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/982), [#996](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/996))

### Ci

- Ingore advisory RUSTSEC-2025-0021 ([!1535](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1535))

## [0.29.1] - 2025-03-27

[0.29.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.29.0...v0.29.1

### 🚀 New features

- (training_participation_report) Add configuration to api ([#972](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/972))
- (training_participation_report) Communicate parameters to frontend on join ([#972](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/972))
- (training_participation_report) Start training participation report procedure automatically ([#972](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/972))

### 🐛 Bug fixes

- Address cargo-deny remarks ([!1480](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1480))
- (timer) Cleanup on room destroy ([#975](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/975))
- Exclude moderators from microphone restrictions ([#978](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/978))
- (event) Handle missing participation report parameter set correctly ([#979](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/979))
- (meeting_report) Include users who already left the meeting ([#981](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/981))
- (training-participation-report) Don't fail when updating the database entry ([#982](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/982))
- (training-participation-report) Parameter set not stored for unscheduled events ([!1507](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1507), [#987](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/987))

### 📦 Dependencies

- (deps) Update ring to 0.17.13 ([!1478](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1478))
- (deps) Update rust crate rand to 0.9 ([!1418](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1418))
- (deps) Update rust crate zip to 2.4.2 ([!1490](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1490))

### ⚙ Miscellaneous

- (justfile) Add commit release script ([!1322](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1322))

### Test

- Use ChaCha12Rng instead of StdRng for reproducibility ([!1480](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1480))

## [0.29.0] - 2025-03-05

[0.29.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.28.4...v0.29.0

### 🚀 New features

- Add short argument & help text for version information ([!1357](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1357))
- Add deprecation warning for `keycloak` setting ([!1348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1348))
- Add an endpoint to determine the readiness of the service (Closes #923) ([!1352](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1352))
- (subroom-audio) Disable whisper functionality by default ([!1374](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1374), [#925](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/925))
- (core) Keep ad-hoc permissions in breakout rooms ([!1381](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1381), [#834](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/834))
- (jobs) Extend event deletion job to cover recurring meetings ([!1407](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1407), [#850](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/850))
- Filter signaling modules for encrypted rooms ([!1422](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1422))
- Add upload endpoint for assets ([!1421](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1421))
- (report) Use `typst` for report generation instead of `terdoc` ([!1344](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1344), [#919](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/919))
- Add created_after and created_before filters to GET /events endpoint ([!1438](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1438), [#949](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/949))
- Add ubuntu based container image ([!1453](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1453))
- (signaling) Implement training participaion report signaling module ([!1441](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1441), [#935](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/935))

### 🐛 Bug fixes

- Workaround bug in `OpenTelemetry` ([!1139](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1139))
- Print readable error message ([!1348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1348))
- Don't print repeated deprecation warnings for `keycloak` setting ([!1348](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1348), [#921](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/921))
- Apply shellcheck lints ([!1370](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1370))
- Restore opentalk-api-client ([!1372](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1372))
- If IPv6 is unavailable on a system, bind to IPv4 only instead of crashing ([!1405](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1405))
- (api) Only include call-in info in API where applicable ([!1420](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1420), [#940](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/940))
- (training-participation-report) Set waiting for initial timeout state when first trainee joins ([!1471](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1471), [#970](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/970))

### 📚 Documentation

- Update migration documentation for livekit release ([!1354](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1354))
- Update migration documentation for oidc config changes ([!1354](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1354))
- Add instructions for generating docs ([!1358](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1358))
- Fix broken links ([!1367](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1367))
- Replace `R2C` with ready status since `R2C` isn't well known or explained earlier ([!1367](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1367))

### 🔨 Refactor

- Remove deprecated `enabled_modules` and `disabled_features` fields from tariff ([!1307](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1307), [#491](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/491))
- Use opentalk-version crate ([!1357](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1357))
- Remove opentalk-smtp-mailer-protocol from the controller repository ([!1380](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1380), [#928](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/928))
- Move ApiError into opentalk-types-api-v1 ([!1382](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1382))
- Remove mail notifications for DELETE /rooms/{room_id} endpoint ([!1412](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1412))
- Pass some global objects to ControllerBackend ([!1378](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1378))
- Make DISPLAY_NAME a global attribute ([!1455](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1455))
- Make IS_ROOM_OWNER a global attribute ([!1455](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1455))

### 📦 Dependencies

- (deps) Update opentelemetry-rs to 0.27 ([!1139](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1139))
- (deps) Update rust crate axum to 0.8 ([!1362](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1362))
- (deps) Update rust crate reqwest to v0.12.12 ([!1359](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1359))
- (deps) Update redocly/cli docker tag to v1.26.1 ([!1356](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1356))
- (deps) Update rust crate itertools to 0.14 ([!1360](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1360))
- (deps) Update rust crate config to 0.15 ([!1355](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1355))
- (deps) Update rust crate chrono-tz to 0.10 ([!1135](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1135))
- (deps) Update rust dependencies utoipa to v5 and utoipa_swagger_ui to v8 ([!1294](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1294))
- (deps) Update redocly/cli docker tag to v1.27.0 ([!1365](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1365))
- (deps) Update redocly/cli docker tag to v1.27.1 ([!1376](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1376))
- (deps) Update redocly/cli docker tag to v1.27.2 ([!1406](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1406))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.84.0 ([!1384](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1384))
- (deps) Update rust crate utoipa-swagger-ui to v9 ([!1408](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1408))
- (deps) Update rust crate validator to 0.20 ([!1409](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1409))
- (deps) Update redocly/cli docker tag to v1.28.0 ([!1423](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1423))
- (deps) Update rust crate service-probe to v0.2.1 ([!1426](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1426))
- (deps) Update redocly/cli docker tag to v1.28.1 ([!1428](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1428))
- (deps) Update redocly/cli docker tag to v1.28.2 ([!1430](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1430))
- (deps) Update redocly/cli docker tag to v1.28.3 ([!1434](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1434))
- (deps) Update rust crate tabled to 0.18 ([!1435](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1435))
- (deps) Update rust crate strum to 0.27 ([!1437](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1437))
- (deps) Update redocly/cli docker tag to v1.28.5 ([!1436](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1436))
- (deps) Update rust crate derive_more to v2 ([!1429](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1429))
- (deps) Update rust crate yaml-rust2 to 0.10.0 ([!1445](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1445))
- (deps) Update opentalk-types-common and opentalk-types-api-v1 ([!1456](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1456))
- (deps) Update typst crates to 0.13 ([!1461](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1461))
- (deps) Update redocly/cli docker tag to v1.31.2 ([!1449](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1449))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.85.0 ([!1465](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1465))
- (deps) Update rust crates redis, redis-args and opentalk-types-* ([!1452](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1452))
- (deps) Update redocly/cli docker tag to v1.31.3 ([!1466](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1466))
- (deps) Update redocly/cli docker tag to v1.32.1 ([!1468](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1468))
- (deps) Update redocly/cli docker tag to v1.32.2 ([!1469](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1469))
- (deps) Update redocly/cli docker tag to v1.33.0 ([!1472](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1472))

### ⚙ Miscellaneous

- (turn) Deprecate turn configuration and endpoint ([!1331](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1331), [#910](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/910))
- Use opentalk-types-* from crates.io ([!1375](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1375), [#926](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/926))
- Update opentalk-types to 0.31 ([!1425](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1425))
- Update dependencies ([!1433](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1433))

### Ci

- No longer allow to fail conventional commit check ([!1363](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1363))
- Only run conventional commit check for merge requests ([!1363](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1363))
- Verify that commits are signed ([!1363](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1363))
- Only require that a commit signature exists ([!1366](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1366))
- Add shellcheck to cli ([!1370](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1370))
- Cargo-deny with all features and deny undetected advisories ([!1371](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1371))

## [0.28.4] - 2025-02-25

[0.28.4]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.28.3...v0.28.4

### 🚀 New features

- Add ubuntu based container image ([!1453](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1453))

## [0.28.3] - 2025-02-03

[0.28.3]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.28.2...v0.28.3

### 📦 Dependencies

- (deps) Update rust crate service-probe to v0.2.1 ([!1426](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1426))

## [0.28.2] - 2025-01-28

[0.28.2]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.28.1...v0.28.2

- Add an endpoint to determine the readiness of the service (Closes #923) ([#923](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/923))

## [0.28.1] - 2025-01-10

[0.28.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.28.0...v0.28.1

### 🚀 New features

- (subroom-audio) Disable whisper functionality by default ([!1374](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1374), [#925](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/925))

## [0.28.0] - 2024-12-12

[0.28.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.27.0...v0.28.0

### 🚀 New features

- Add subroom audio module for whisper functionality ([!1269](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1269), [#872](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/872))
- (openapi) Normalize whitespace in output of `openapi dump` subcommand ([!1339](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1339), [#912](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/912))

### 🐛 Bug fixes

- (types) Properly build the axum response from the ApiError ([!1336](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1336))
- Use `snake_case` for `ReportTemplate` setting ([!1341](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1341))

### 📚 Documentation

- Update meeting-report template description ([!1341](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1341), [#918](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/918))

### 🔨 Refactor

- (types) Introduce opentalk-types-api-v1 crate ([!1335](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1335), [#881](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/881))
- (types) Remove deprecated POST /services/recording/upload_render endpoint ([!1335](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1335))

### 📦 Dependencies

- (deps) Do not skip num-bigint & ordered-float with cargo-deny ([!1346](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1346))
- (deps) Lock file maintenance ([!1333](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1333))
- (deps) Update alpine docker tag to v3.21 ([!1343](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1343))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.83.0 ([!1340](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1340))
- (deps) Update redocly/cli docker tag to v1.26.0 ([!1345](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1345))
- (deps) Update rust crate cargo_metadata to 0.19 ([!1333](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1333))
- (deps) Update rust crate sysinfo to 0.33 ([!1342](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1342))
- (deps) Update rust crate tabled to 0.17 ([!1333](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1333))
- (deps) Update rust crate validator to 0.19 ([!1333](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1333))

### ⚙ Miscellaneous

- Fix clippy lints for rustc 1.83 ([!1338](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1338))

### Test

- Test meeting report settings ([!1341](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1341))

## [0.27.0] - 2024-11-20

[0.27.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.26.0...v0.27.0

### 🚀 New features

- (livekit) Add signaling for a popout stream access token ([!1312](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1312), [#901](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/901))

### 🔨 Refactor

- Move all crates into paths matching the crate name ([!1325](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1325))
- (types) Move NamespacedCommand to opentalk-types-signaling ([!1326](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1326))
- (types) Move NamespacedEvent to opentalk-types-signaling ([!1326](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1326))
- (types) Remove core signaling module from opentalk-types ([!1326](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1326))
- (types) Remove echo signaling module from opentalk-types ([!1326](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1326))
- (types) Remove integration signaling module from opentalk-types ([!1326](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1326))

### Ci

- Only run ci jobs for types crates when needed ([!1324](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1324))

## [0.26.0] - 2024-11-13

[0.26.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.25.0...v0.26.0

### 🐛 Bug fixes

- Skip the rooms grace period when the controller is shutdown ([!1317](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1317))
- (streaming) Hide streaming key for users other than room creator/owner ([!1318](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1318))

### 📚 Documentation

- Add LiveKit migration guide ([!1302](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1302))
- Remove frontend migration since it's not necessary anymore ([!1320](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1320))
- Add LiveKit Signaling Module documentation ([!1316](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1316))
- Update documentation for LiveKit, removing support for Janus ([!1303](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1303))

### 📦 Dependencies

- Lock file maintenance ([!1314](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1314))
- Update livekit-api to 0.4.1, livekit-protocol to 0.3.5, livekit-runtime to 0.3.1 ([!1319](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1319), [#903](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/903))
- Update redocly/cli docker tag to v1.25.11 ([!1288](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1288))
- Update rust crate diesel-async to v0.5.1 ([!1311](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1311))
- Update rust crate opentalk-etherpad-client to 0.2.0 ([!1279](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1279))

### ⚙ Miscellaneous

- Fix redundant `the` in comments ([!1316](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1316))
- (media) Remove janus-media signaling module and types ([!1303](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1303), [#896](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/896))

## [0.25.0] - 2024-10-30

[0.25.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.21.0...v0.25.0

### 🚀 New features

- (types) Types-signaling-livekit crate & send urls to services ([!1274](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1274))
- Add enable/disable microphone restrictions ([!1268](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1268))
- (jobs) Add user request page size in keycloak-account-sync job ([!1227](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1227), [#875](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/875))
- (livekit) Notify participant about force mute via signaling ([!1298](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1298), [#892](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/892))
- (core) Add grace period to room destruction ([!1238](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1238), [##833](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/#833))

### 🐛 Bug fixes

- (docs) Add `endpoints.disable_openapi` to the endpoints documentation ([!1242](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1242), [#889](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/889))
- (docs) Fix invalid example json ([!1245](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1245))
- Wrong utoipa names ([!1247](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1247))
- (recording) Avoid calling HMSET when no streams are configured ([!1233](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1233), [#867](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/867))
- (docs) Invite resource API documentation ([!1253](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1253), [#132](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/132))
- (ci) Add openapi spec check and fix issues in generation ([!1253](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1253))
- Shebang with /usr/bin/env bash ([!1275](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1275))
- (docs) Add default values for service.name and service.namepsace to tracing docs ([!1273](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1273))

### 🔨 Refactor

- (types) Introduce opentalk-types-signaling-recording crate ([!1236](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1236), [#880](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/880))
- (types) Introduce opentalk-types-signaling-recording-service crate ([!1256](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1256), [#879](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/879))
- (types) Introduce opentalk-types-signaling-chat crate ([!1272](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1272), [#871](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/871))
- (types) Introduce opentalk-types-signaling-moderation crate ([!1277](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1277), [#887](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/887))
- (types) Introduce opentalk-types-signaling-polls crate ([!1286](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1286), [#886](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/886))
- (types) Introduce opentalk-types-signaling-meeting-notes crate ([!1290](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1290), [#884](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/884))
- (types) Introduce opentalk-types-signaling-shared-folder crate ([!1295](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1295), [#883](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/883))
- (types) Introduce opentalk-types-signaling-timer crate ([!1297](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1297), [#882](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/882))
- (types) Introduce opentalk-types-signaling-whiteboard crate ([!1301](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1301), [#885](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/885))
- (config) Rework OIDC and user search configuration ([!1209](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1209))
- Add a cleanup scope to the destroy_context ([!1238](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1238))
- (types) Introduce opentalk-types-signaling-meeting-report crate ([!1304](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1304), [#897](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/897))

### 📦 Dependencies

- (deps) Update rust crate aws-sdk-s3 to v1.55.0 ([!1234](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1234))
- (deps) Lock file maintenance ([!1240](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1240))
- (deps) Update redocly/cli docker tag to v1.25.6 ([!1241](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1241))
- (deps) Update rust crate uuid to v1.11.0 ([!1248](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1248))
- (deps) Update rust crate rustls-pki-types to v1.10.0 ([!1246](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1246))
- (deps) Update redocly/cli docker tag to v1.25.7 ([!1251](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1251))
- (deps) Update rust crate proc-macro2 to v1.0.88 ([!1252](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1252))
- (deps) Update rust crate serde_json to v1.0.129 ([!1255](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1255))
- (deps) Update rust crate aws-sdk-s3 to v1.57.0 ([!1254](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1254))
- (deps) Update rust crate redis to v0.27.5 ([!1257](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1257))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.82.0 ([!1258](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1258))
- (deps) Lock file maintenance ([!1263](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1263))
- (deps) Update rust crate fern to 0.7 ([!1262](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1262))
- (deps) Update rust crate bytes to v1.8.0 ([!1265](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1265))
- (deps) Update redocly/cli docker tag to v1.25.8 ([!1264](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1264))
- (deps) Update rust crate serde to v1.0.211 ([!1266](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1266))
- (deps) Update rust crate tokio to v1.41.0 ([!1267](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1267))
- (deps) Update rust crate serde to v1.0.213 ([!1270](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1270))
- (deps) Update rust crate proc-macro2 to v1.0.89 ([!1271](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1271))
- (deps) Update rust crate config to v0.14.1 ([!1278](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1271))
- (deps) Update rust crate syn to v2.0.85 ([!1276](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1276))
- (deps) Update rust crate opentalk-nextcloud-client to 0.2.0 ([!1280](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1280))
- (deps) Update rust crate anstream to v0.6.17 ([!1287](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1287))
- (deps) Lock file maintenance ([!1292](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1292))
- (deps) Update rust crate bigdecimal to v0.4.6 ([!1293](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1293))
- (deps) Update rust crate serde to v1.0.214 ([!1299](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1299))

### Ci

- Post changelog info ([!1237](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1237))

## [0.21.0]

[0.21.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.20.0...v0.21.0

### 🚀 New features

- Generate attendance report ([!1074](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1074), [#558](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/558))
- Add `ToSchema` derive to `ModuleFeatureId` ([!1210](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1210))
- Don't print warning when skipping modules ([!1201](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1201))
- Warn about adding rules twice ([!1201](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1201))
- Add force mute command to livekit module ([!1200](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1200))
- Respect defaults.screen_share_requires_permission & add grant/revoke_screen_share_permissions ([!1200](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1200))
- (release) Add a `justfile` with a `prepare-release` target for release automation ([!1226](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1226))

### 🐛 Bug fixes

- (recording) Rollback object storage after certain save asset errors ([!1132](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1132), [#860](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/860))
- Deserialize errors on missing fields ([!1187](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1187))
- Properly serialize url queries ([!1187](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1187))
- Cleanup permissions after removing user from event ([!1201](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1201), [#849](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/849))
- Properly sync profile pictures on login ([!1224](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1224), [#852](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/852))

### 🔨 Refactor

- (types) Introduce opentalk-types-signaling-breakout crate ([!1177](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1177), [#868](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/868))
- (types) Remove wildcard `imports` modules in type crates ([!1205](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1205))
- (types) Introduce opentalk-types-signaling-control crate ([!1204](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1204), [#870](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/870))

### 📦 Dependencies

- (deps) Lock file maintenance ([!1183](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1183), [!1198](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1198), [!1223](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1223))
- (deps) Update git.opentalk.dev:5050/opentalk/tools/check-changelog docker tag to v0.3.0 ([!1185](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1185))
- (deps) Update postgres docker tag to v17 ([!1193](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1193))
- (deps) Update redocly/cli docker tag to v1.25.5 ([!1216](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1216))
- (deps) Update rust crate async-trait to v0.1.83 ([!1189](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1189))
- (deps) Update rust crate aws-sdk-s3 to v1.53.0 ([!1211](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1211))
- (deps) Update rust crate cidr to 0.3 ([!1202](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1202))
- (deps) Update rust crate clap to v4.5.20 ([!1228](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1228))
- (deps) Update rust crate http-request-derive to v0.3.2 ([!1212](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1212))
- (deps) Update rust crate opentalk-diesel-newtype to 0.13 ([!1206](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1206))
- (deps) Update rust crate proc-macro2 to v1.0.87 ([!1225](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1225))
- (deps) Update rust crate redis to v0.27.4 ([!1229](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1229))
- (deps) Update rust crate rustls-pemfile to v2.2.0 ([!1203](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1203))
- (deps) Update rust crate serde_with to v3.10.0 ([!1208](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1208))
- (deps) Update rust crate snafu to v0.8.5 ([!1186](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1186))
- (deps) Update rust crate sysinfo to 0.32 ([!1222](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1222))
- (deps) Update rust crate tokio-cron-scheduler to 0.13 ([!1182](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1182))
- (deps) Update rust crate yaml-rust2 to 0.9.0 ([!1191](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1191))

### ⚙ Miscellaneous

- Return livekit errors in snake_case ([!1188](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1188))
- Fix typo in rooms e2e_encryption field ([!1190](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1190))
- (ci) Extract crate checks into included gitlab-ci.yml files ([!1205](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1205))
- (tests) Rename `mod test` to `mod tests` ([!1205](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1205))

### Ci

- Remove changelog-check ([!1178](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1178))

### Test

- (kustos) Verify granting access again works ([!1201](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1201))

## [0.20.0]

[0.20.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.19.0...v0.20.0

### 🚀 New features

- Return ack messages for moderator and presenter changes ([!1103](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1103))
- (moderation) Improve signaling responses for the `ChangeDisplayName` command ([!1119](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1119))
- Add livekit module ([!1063](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1063))
- Make janus-media module optional ([!1063](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1063))
- Add e2e encryption flag to rooms table ([!1124](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1124))

### 🐛 Bug fixes

- Prevent high cpu usage when RabbitMQ is unavailable ([!1125](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1125))
- Wrong documented response body of /rooms/{room_id}/event ([!1126](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1126))
- Always include streaming_links property in MeetingDetails ([!1128](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1128))
- Change the WWW-Authenticate error value to `invalid_token` for expired sessions ([!1134](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1134))
- (protocol) Rename protocol module to meeting-notes ([!1004](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1004))
- Remove the `is_room_owner` key on room cleanup ([!1131](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1131))
- (signaling) Correctly serialize/deserialize namespaced messages ([!1166](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1166))

### ⚙ Miscellaneous

- Return livekit errors in snake_case ([!1188](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1188))

### 📚 Documentation

- Add diagram and descriptions for the participant lifecycle ([!1144](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1144))

### 🔨 Refactor

- (types) Introduce opentalk-types-common crate ([!1137](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1137))
- (types) Introduce opentalk-types-signaling crate ([!1137](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1137))

### 📦 Dependencies

- (deps) Lock file maintenance ([!1116](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1116))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.81.0 ([!1136](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1136))
- (deps) Update git.opentalk.dev:5050/opentalk/tools/check-changelog docker tag to v0.2.0 ([!1143](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1143))
- (deps) Update rabbitmq docker tag to v4 ([!1175](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1175))
- (deps) Update redocly/cli docker tag to v1.25.3 ([!1174](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1174))
- (deps) Update rust crate async-trait to v0.1.82 ([!1156](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1156))
- (deps) Update rust crate aws-sdk-s3 to v1.51.0 ([!1172](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1172))
- (deps) Update rust crate bytes to v1.7.2 ([!1173](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1173))
- (deps) Update rust crate clap to v4.5.17 ([!1146](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1146))
- (deps) Update rust crate diesel to v2.2.4 ([!1147](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1147))
- (deps) Update rust crate gix-path to 0.10.11 ([!1138](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1138))
- (deps) Update rust crate owo-colors to v4.1.0 ([!1161](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1161))
- (deps) Update rust crate pretty_assertions to v1.4.1 ([!1168](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1168))
- (deps) Update rust crate redis to 0.26 & redis-args to 0.16 ([!1067](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1067))
- (deps) Update rust crate redis-args to 0.17 ([!1169](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1169))
- (deps) Update rust crate rrule to 0.13 ([!1081](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1081))
- (deps) Update rust crate serde_json to v1.0.128 ([!1152](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1152))
- (deps) Update rust crate syn to v2.0.77 ([!1153](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1153))
- (deps) Update rust crate sysinfo to v0.31.4 ([!1158](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1158))
- (deps) Update rust crate tokio to v1.40.0 ([!1164](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1164))
- (deps) Update rust crate tokio-stream to v0.1.16 ([!1154](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1154))
- (deps) Update rust crate vergen to v9.0.1 ([!1170](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1170))
- (deps) Update rust crate vergen-gix to v1.0.2 ([!1171](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1171))

### ⚙ Miscellaneous

- (dependencies) Update crate gix-path to fix RUSTSEC-2024-0367 ([!1122](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1122))
- Update default terdoc port ([!1123](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1123))
- Ignore RUSTSEC-2024-0370 ([!1130](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1130))
- Upgrade redocly/cli image ([!1127](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1127))
- Fix redis related clippy lints ([!1067](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1067))
- Add snafu::report to xtask ([!1124](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1124))

### Ci

- Check changelog ([!1115](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1115))

### Test

- Enhanced unit test for update message ([!1103](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1103))

## [0.19.1]

[0.19.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.19.0...v0.19.1

### 🐛 Bug fixes

- Always include `streaming_links` property in `MeetingDetails` ([#856](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/856), [!1128](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1128))

### 📦 Dependencies

- Update rust crate gix-path to 0.10.10 (fixing [RUSTSEC-2024-0367](https://rustsec.org/advisories/RUSTSEC-2024-0367.html)) ([!1122](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1122))

## [0.19.0]

[0.19.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.18.0...v0.19.0

### 🚀 New features

- Add part-number for chunk upload ([!1086](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1086))
- Serve a OpenAPI Swagger page under `/swagger` ([#759](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/759), [!828](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/828))
- Add subcommand for exporting the OpenAPI specification to stdout or a file ([#759](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/759), [!828](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/828))

### 🐛 Bug fixes

- Prevent recorder start in breakout rooms ([#838](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/838), [!1094](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1094))
- Clippy v1.80.0 lints ([!1073](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1073))
- Interior mutability issue as reported by clippy ([!1073](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1073))
- (jobs) Fix storage sync bug where a low amount of assets resulted in a job failure ([#842](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/842), [!1114](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1114))
- (ci) Ignore frontend api yaml file based on name ([!1120](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1120))
- (ci) Update markdown linter to allow code blocks ([!1113](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1113))
- (ci) Detect only .rs files instead of anything ending on rs ([!1118](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1118))

### 📚 Documentation

- How to setup the recorder client in Keycloak ([#817](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/817), [!1105](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1105))

### 📦 Dependencies

- Update rust crate clap to v4.5.14 ([!1092](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1092))
- Update rust crate derive_more to v1 ([!1087](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1087))
- Update rust crate diesel-async to 0.5 ([!631](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/631))
- Update rust crate serde to v1.0.207 ([!1101](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1101))
- Update rust crate serde_json to v1.0.124 ([!1102](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1102))
- Update rust crate tokio-cron-scheduler to 0.11 ([!1095](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1095))

### 🔨 Refactor

- Use `BTree{Map,Set}` in module features for more stable (de-)serialization ([!828](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/828))
- Move GET `/rooms/{room_id}/invites` response to separate struct ([!828](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/828))
- Create RecurrencePattern and RecurrenceRule newtypes ([!828](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/828))

### ✨ Style

- Use consistent module file layout ([!911](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/911))

### 🐛 Bug fixes

- Rollback object storage after certain save asset errors ([#860](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/860))

## [0.18.0]

[0.18.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.17.0...v0.18.0

### 🚀 New features

- Use avatar url if the JWT `picture` claim is set ([#824](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/824))

### 🐛 Bug fixes

- Override streaming_targets on PATCH '/events/{event_id}' ([#829](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/829))

### 📚 Docs

- Add documentation for HTTP request handling ([#826](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/826))
- Add documentation for OIDC auth ([#826](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/826))
- Document the JWT `picture` field ([#824](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/824))

### 🔨 Dependencies

- Update opentelemetry implementation
- Update opentelemetry-rs
- Update rust crate anstream to v0.6.15
- Update rust crate aws-sdk-s3 to v1.42.0
- Update rust crate bytes to v1.7.1
- Update rust crate clap to v4.5.13
- Update rust crate email_address to v0.2.9
- Update rust crate env_logger to v0.11.5
- Update rust crate lapin to v2.5.0
- Update rust crate rustls to v0.23.12
- Update rust crate rustls-pemfile to v2.1.3
- Update rust crate rustls-pki-types to v1.8.0
- Update rust crate serde_json to v1.0.122
- Update rust crate syn to v2.0.72
- Update rust crate sysinfo to v0.31.2
- Update rust crate tabled to 0.16
- Update rust crate tokio to v1.39.2
- Update rust crate vergen to v9

## [0.17.0]

[0.17.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.16.0...v0.17.0

### 🚀 New features

- Syncronize ACL changes via rabbitmq between controllers ([!997](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/997))
- Add configuration for terdoc report generation service ([#1035](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/815))
- Check openapi specification with stoplight spectral ([!1032](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1032))
- Add axum compatibility for the ApiError ([#808](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/808))
- Allow recorder to join breakout rooms ([#804](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/804))

### 🐛 Bug fixes

- Delete room assets on event deletion ([!977](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/977))
- Clean up force mute state when meeting is closed ([#812](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/812))
- Specify usage of the serde feature for the opentalk-types dependency ([#1049](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1049))

### ⚙ Miscellaneous

- Update rust crate async-trait to v0.1.81
- Update rust crate aws-sdk-s3 to v1.41.0
- Update rust crate bytes to v1.6.1
- Update rust crate clap to v4.5.9
- Update rust crate email_address to v0.2.5
- Update rust crate lapin to v2.4.0
- Update rust crate log to v0.4.22
- Update rust crate moka to v0.12.8
- Update rust crate phonenumber to v0.3.6
- Update rust crate redis-args to 0.15
- Update rust crate serde to v1.0.204
- Update rust crate serde_json to v1.0.120
- Update rust crate serde_with to v3.9.0
- Update rust crate snafu to v0.8.4
- Update rust crate syn to v2.0.71
- Update rust crate sysinfo to v0.30.13
- Update rust crate tokio to v1.38.1
- Update rust crate uuid to v1.10.0
- Update rust crate vergen to v8.3.2

### 📚 Docs

- Add mail worker protocol schema and examples ([#811](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/811))
- Document quota types ([#768](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/768))

### 🔨 Refactor

- Introduce enum for quota types ([!1026](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1026))

## [0.16.1]

### :bug: Bug fixes

- Properly delete room assets on event deletion ([!977](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/977))

## [0.16.0]

[0.16.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.15.0...v0.16.0

### <!-- 0 -->:rocket: New features

- Allow connecting to janus via websocket_url [#786](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/786)
- Add room and room-creator info to the join success message ([#779](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/779))
- Add job to sync account states ([#776](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/776))
- Extend DELETE rooms & events endpoints, remove `DELETE /internal/rooms/{room_id}` endpoint ([#762](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/762))
- Force disable microphones of participants ([#711](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/711))
- Allow lowering of multiple raised hands with a single command ([#790](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/790))
- Add shared folder option to PATCH /v1/events/{event_id} ([#784](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/784))
- Add streaming target option to PATCH /v1/events/{event_id} ([#784](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/784))
- Extend moderator mute so that backend can mute all participants ([#798](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/798))
- Add job for deleting disabled users ([#777](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/777))
- Include room-id in ticket token ([!1000](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1000))
- Do not use rabbitmq for exchange when no redis is configured ([!1001](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/1001))

### <!-- 1 -->:bug: Bug fixes

- Also check the current directory for .git files or folders ([!948](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/948))
- Move event_loops field out of janus connection configuration enum ([#791](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/791))
- Handle undefined values in volatile storage ([#789](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/789))
- Set joined time when joining ([#797](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/797))
- Enable serde derives for `serde` feature instead of `client` ([#799](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/799))
- Show correct package version and optimization level ([!946](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/946))
- Fix speaker detection by assuming the unmute is allowed if no unmute allowlist is present and by updating the speaker state when already initialized ([#801](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/801))
- Update poll error documentation ([#781](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/781))

### Ci

- Add lint script for detecting modules that should use mod.rs files
- Call `cargo-deny` with `--deny unmatched-skip` ([#803](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/803))

### Docs

- Describe required Keycloak settings for enabling user search ([#729](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/729))

### Refactor

- Add OneOrManyVec and OneOrManyBTreeSet types

### :gear: Miscellaneous

- Use opentalk-keycloak-admin from crates.io
- Fix clippy lints for rustc 1.79.0

### Dependencies

- Update alpine docker tag to v3.20
- Update curve25519-dalek to fix RUSTSEC-2024-0344
- Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.79.0
- Update postgres docker tag to v16
- Update rabbitmq docker tag to v3.13
- Update redis docker tag to v7
- Update rust crate actix to v0.13.5
- Update rust crate actix-http to v3.8.0
- Update rust crate actix-rt to v2.10.0
- Update rust crate actix-web to v4.8.0
- Update rust crate actix-web-httpauth to v0.8.2
- Update rust crate aws-sdk-s3 to v1.38.0
- Update rust crate bigdecimal to v0.4.5
- Update rust crate cidr to v0.2.3
- Update rust crate clap to v4.5.8
- Update rust crate derive_more to v0.99.18
- Update rust crate either to v1.13.0
- Update rust crate proc-macro2 to v1.0.86
- Update rust crate rustc-hash to v2
- Update rust crate serde_json to v1.0.119
- Update rust crate serde_with to v3.8.2
- Update rust crate strum to v0.26.3
- Update rust crate syn to v2.0.68
- Update rust crate url to v2.5.2
- Update rust crate uuid to v1.9.1

## [0.15.0]

[0.15.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.14.0...v0.15.0

### :rocket: New features

- controller: Allow resetting individual participant's raised hands ([#764](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/764))
- mail-worker-protocol: add streaming targets ([#650](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/650))
- assets: Save assets in a predefined name format ([#763](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/763))
- controller: keep signaling open when sending user from room to waiting room ([#740](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/740))
- Include `show_meeting_details` in POST, PATCH and GET Event ([#769](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/769))
- Send error in case of insufficient permissions ([!890](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/890))
- Add job to synchronize database assets and storage files ([#665](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/665))
- Add job to cleanup orphaned rooms ([#727](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/727))
- Add 'disabled_since' flag to users & filter disabled users ([#775](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/775))
- Add in memory alternative to redis ([!895](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/895))

### :bug: Bug fixes

- media: fix speaker detection by updating the speaker state when already initialized ([#801](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/801))
- mail-worker-protocol: Enable serde derives for `serde` feature instead of `client` ([#799](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/799))
- dep: Update curve25519-dalek to fix RUSTSEC-2024-0344
- Update rust crate proc-macro2 to v1.0.83
- Update rust crate nix to 0.29
- Update rust crate actix-http to v3.7.0
- Update rust crate proc-macro2 to v1.0.84
- Update rust crate proc-macro2 to v1.0.85
- Update rust crate etcd-client to 0.13
- Update rust crate tracing-actix-web to v0.7.11
- Add notification mail to internal room deletion ([#720](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/720))
- Inconsistent features
- Update rust crate tokio-cron-scheduler to v0.10.2
- Cleanup closed poll list ([!895](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/895))

### Docs

- Add manual for deleting a user from the database ([#774](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/774))
- Document configuration changes regarding redis

## [0.14.0]

[0.14.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.13.0...v0.14.0

### :rocket: New features

- recording: make `record` and `stream` functionality configurable by module features ([#760](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/760))
- controller: Allow polls with multiple choices ([#746](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/746))
- Add a distributed JobExecutor system ([#422](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/422), [#424](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/424), [#425](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/425))

### :bug: Bug fixes

- database: Make events.room unique to create one to one relation ([#724](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/724))
- Add missing and underspecified asset information
- controller: only notify once about enabled/disabled waiting room ([#757](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/757))

### :gear: Miscellaneous

- Update mail-worker-protocol metadata for publishing to crates.io ([#728](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/728))
- Use lapin-pool from crates.io

### Refactor

- Remove dependency from mail-worker-protocol to db-storage and keycloak-admin ([#754](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/754))

### Ci

- Enforce conventional commits

## [0.13.0]

[0.13.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.12.1...v0.13.0

### Added

- controller: Add a websocket-based asset upload interface (currently used for recordings) ([#614](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/614))
- controller: Add the ability to show meeting details for a room to all participants ([#723](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/723))
- controller: `reason` field in `opentalk-types::signaling::control::Left` ([#741](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/741))

### Changed

- controller: upgrade debian image in ci & container creation to bookworm ([#742](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/742))
- controller: improve output structure when an error is encountered ([#748](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/748))

### Fixed

- controller: display names longer than 100 bytes are rejected ([#744](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/744))

## [0.12.1]

[0.12.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.12.0...v0.12.1

### Fixed

- [RUSTSEC-2024-0336](https://rustsec.org/advisories/RUSTSEC-2024-0336)

## [0.12.0]

[0.12.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/da834e3e401c6a9e3e3d03c1d77ff7ff758f6e23...v0.12.0

### Fixed

- [RUSTSEC-2024-0332](https://rustsec.org/advisories/RUSTSEC-2024-0332)

### Added

- controller: Add signaling messages to send participants to the waiting room ([#709](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/709))
- controller: Add the `change_display_name` command to change the display name of a guest or phone user ([#721](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/721))

## [0.11.0]

[0.11.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/61a936a1a88a63804a2b8cfa3d602cb941ef3944...v0.11.0

### Added

- controller: set & enforce maximum storage via `max_storage` quota ([#651](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/651))
- controller: add the option to specify the role of email users when they are invited to an event ([#661](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/661))
- controller: Add API endpoint to query assets associated with a user ([#737](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/737))

### Fixed

- types: don't serialize fields in media state if their value would be `null` ([#716](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/716))

## [0.11.0-rc.2]

### Fixed

- types: don't serialize fields in media state if their value would be `null` ([#716](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/716))

## [0.11.0-rc.1]

### Added

- controller: set & enforce maximum storage via `max_storage` quota ([#651](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/651))
- controller: add the option to specify the role of email users when they are invited to an event ([#661](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/661))
- controller: Add API endpoint to query assets associated with a user ([#737](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/737))

## [0.10.0]

[0.10.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/5ffe66a5586f6792c809a9abefc6023db2e2687a...v0.10.0

### Added

- controller: add streaming and shared folder information to POST /v1/events ([#652](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/652))
- controller: update user related cache entry after calling `PATCH /users/me` ([#660](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/660))
- controller: send update mails for changes to streaming targets and shared folder ([#653](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/653))

### Fixed

- controller: improve error message if signaling protocol header is not valid or missing.

## [0.9.1] - 2024-03-13

### Fixed

- set is_speaking to false on participant mute ([#663](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/663))

## [0.9.0] - 2024-02-22

[0.9.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/1ef2d3091f427c258266a968aa2ffdc5116cc0af...v0.9.0

### Added

- controller: add endpoints for storing room-related streaming targets ([#601](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/601))
- mail-worker-protocol: create event update mail tasks when an event instance gets updated ([#504](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/504))
- controller: add status filter to event invites endpoint ([#610](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/610))
- controller: add reply to hand raise and hand lower ([#624](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/624))

### Changed

- db-storage: add migration to remove `UTIL=XXX` from `recurrence_pattern` field in `events` ([#616](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/616))
- controller/janus-media: let clents communicate their speaking state instead using the detection by janus ([#538](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/538))

### Fixed

- controller: fixed a bug where the configured `default_directives` in the `logging` section could not overwrite the controllers default values ([#582](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/582))
- controller: fixed a bug where event instance ID parsing was failing for the `patch` event instance endpoint ([#631](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/631))
- fix(deps): RUSTSEC-2024-0003 by updating `h2` to `0.3.24` ([#645](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/645))

## [0.8.0-rc.1] - 2024-01-11

### Added

- controller: add endpoints for storing room-related streaming targets ([#601](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/601))

### Changed

- db-storage: add migration to remove `UTIL=XXX` from `recurrence_pattern` field in `events` ([#616](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/616))

### Fixed

- controller: fixed a bug where the configured `default_directives` in the `logging` section could not overwrite the controllers default values ([#582](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/582))
- controller: fixed a bug where event instance ID parsing was failing for the `patch` event instance endpoint ([#631](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/631))

## [0.7.1] - 2024-01-10

[0.7.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.7.0...v0.7.1

### Added

- mail-worker-protocol: added `adhoc_retention_seconds` field to `Events`([#591](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/591))
- controller: added `external_id_user_attribute_name` setting used for searching Keycloak users ([#609](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/609))

### Fixed

- kustos: Do not exit load policy task if it fails once ([!615](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/615))
- chore: ignore RUSTSEC-2023-0071 ([!621](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/621))
- chore: update zerocopy to 0.7.31 ([!622](https://git.opentalk.dev/opentalk/backend/services/controller/-/merge_requests/622))

### Changed

- controller: increase user search limit to 20 from 5 ([#596](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/596))
- controller: don't send e-mail notification to creators of ad-hoc meetings ([#606](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/606))

## [0.7.0] - 2023-10-30

[0.7.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/a79a32ead8943a1e0ecee9b34ecaabdf495b6112...v0.7.0

### Added

- controller: send invite, update and cancellation mails also to creator of event ([#563](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/563))
- controller: add a settings option for the name of the tenant id Keycloak user attribute ([#463](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/463))
- controller: send emails to users when they are removed from a meeting ([#480](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/480))
- jobs: add subcommand to show default job parameters ([#500](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/480))
- controller: add the option to specify the role of registered users when they are invited to an event ([#507](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/507))
- polls: allow poll choice modification ([#508](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/508))
- controller: add endpoint to withdraw invites to participants that were invited via email ([#499](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/499))
- controller: add `is_adhoc` flag in meeting event information ([#554](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/554))
- jobs: add job for cleanup of expired invites ([#506](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/506))

### Changed

- controller/janus-media: Allow the controller to start even though not all Janus instances are available ([#553](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/553))
- shared_folder: use passwords generated by NextCloud, no longer generating them in the Controller ([#539](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/539))
- controller: handle email addresses in a case-insensitive way ([#550](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/550))
- recording: remove consent after the recording has stopped or leaving the room ([#565](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/565))
- controller: increase user search limit to 20 from 5 ([#596](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/596))
- controller: don't send e-mail notification to creators of ad-hoc meetings ([#606](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/606))
- db-storage: add migration to remove `UTIL=XXX` from `recurrence_pattern` field in `events` ([#616](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/616))

### Fixed

- controller: fixed a bug where no emails were sent when deleting an event not having an end date ([#498](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/498))
- controller: fix deletion of permissions to room and event when a registered user gets uninvited ([#543](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/543))
- controller: fixed a bug where waiting room users were displayed as in meeting room ([#542](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/542))
- controller: fixed a bug where participants might circumvent the waiting room when rejoining ([#549](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/549))
- keycloak-admin: log information when Keycloak returns error responses ([#568](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/568))
- chore: fix RUSTSEC-2023-0065 ([#572](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/572))
- chore: fix RUSTSEC-2023-0052 (part 2) ([#571](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/571))
- controller: fixed a bug where deleting a room or an event has failed due to wrong permission checks ([#569](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/569))

## [0.6.3] - 2023-09-26

### Added

- controller: add the option to specify the role of registered users when they are invited to an event ([#507](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/507))

### Changed

- controller: handle email addresses in a case-insensitive way ([#550](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/550))

## [0.6.2] - 2023-09-12

### Fixed

- Add missing release section in CHANGELOG.md

## [0.6.1] - 2023-09-12

### Fixed

- RUSTSEC-2023-0052

## [0.6.0] - 2023-31-08

[0.6.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/32b4c96e3ad319c95baebdfa075d23543b38a8f2...v0.6.0

### Added

- controller: add the capability to disable specific features in the config file or via a tariff ([#394](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/394))
- controller: add the possibility to disable the `call-in` feature via config or tariff ([#395](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/395))
- controller: add a settings option to prohibit the user from changing the display name ([#415](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/415))
- controller: expose enabled features to the frontend (and make them module-specific) ([#471](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/471))
- jobs: add subcommand to execute maintenance jobs from the comnand-line interface ([#486](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/486))

### Changed

- controller: changed the error messages for invalid configurations to be verbose and include the full path to the missing/invalid field ([#465](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/465))

### Fixed

- controller: fixed a bug where a response from the REST API was missing CORS information when an invalid access token was provided ([#436](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/436))
- controller: fixed some issues related to the timer ready state reported to the frontend ([#411](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/411)
- controller: fixed a bug where a debrief without enough participants led to an error ([#429](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/429))
- controller: for static tenant setting, no longer filters users by tenant when searching them by email ([#469](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/469))
- controller: fixed a bug where the V27 migration could not be applied when legal-votes without an associated room exist ([#494](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/494))
- controller: fixed a bug where no emails were sent when deleting an event not having an end date ([#498](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/498))

## [0.5.0] - 2023-06-27

[0.5.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/2129f33efb8898d46f6dd1b43db43e5cbb929f99...v0.5.0

### Added

- controller: add tariff status and handle downgraded tariff ([#408](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/408))
- controller: extend `JoinSuccess` signaling message's `Event` with `EventId` field ([#399](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/399))
- controller: add event information to `JoinSuccess` signaling message ([#266](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/266))
- controller: add the `shared-folder` module, allowing users to create and connect a nextcloud share in their conferences ([#381](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/381))

### Changed

- logging: `RUST_LOG` environment variable entries override settings from configuration file ([#69](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/69))
- controller: don't send mail notifications on deletion of past events ([#407](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/407))
- controller: `x_grp` defaults to empty if not provided ([#414](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/414))
- controller: respect operating system CA certificates for all outgoing tls connections ([#382](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/382))
- controller: fix signaling for rooms without events ([406](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/406))
- db-storage/mail-worker-protocol: added `revision` field to `events` to track the number of changes
- cli: Update `fix-cli` subcommand, now also fixes access to events and legal-votes ([#387](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/387))

### Fixed

- controller: Avoid sending unnecessary close frames. ([#356](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/356))
- janus-media: discard unhandled ack messages, log them on debug level only ([#252](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/252))
- cli: fix a bug where the `fix-acl` command was not working when too many permissions were added ([403](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/403))
- signaling: Consider the `enable_phone_mapping` config value when trying to match the phone number to a opentalk user

## [0.4.0] - 2023-05-25

[0.4.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/60b8af4daa0e0f2ed2ec9589fd1c9da3218baf8c...v0.4.0

### Added

- controller: cache access-token check results for a maximum of 5mins, reducing load on both Keycloak and postgres ([#359](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/359))
- janus-media: add `event_loops` options to specify how many event-loop the janus instance runs on. Used to send hints to janus on which event-loop to create a new webrtc-session (handle).
- controller: add debriefing and kicking multiple users at once ([#350](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/350))
- controller: always allow one moderator to join a room regardless of participant limit ([#352](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/352))
- chat: add private chat history ([#327](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/327))
- controller: kick users when the owner deletes the room ([#328](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/328))

### Changed

- naming: replace initial project code name `k3k` by `opentalk` in code, executable names and environment variables ([#279](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/279))
- controller: respect operating system CA certificates for all outgoing tls connections ([#382](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/382))

### Fixed

- controller/db-storage: Email invites now get deleted, when converted to user invites. ([#320](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/320))
- controller-settings: Fixed a panic when trying to parse config values for `tenants` and `tariffs`, when their assignment was set to `static`
- controller: Avoid sending unnecessary close frames. ([#356](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/356))

### Moved

- types: Move `Namespaced` to `types::signaling::NamespacedCommand`
- types: Move `NamespacedOutgoing` to `types::signaling::NamespacedEvent`

## [0.3.1] - 2023-04-24

[0.3.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.3.0...v0.3.1

### Fixed

- `(backport):` controller-settings: Fixed a panic when trying to parse config values for `tenants` and `tariffs`, when their assignment was set to `static`

## [0.3.0] - 2023-04-17

[0.3.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/382d6f2d1ccac530431a1fe7f8379ed21769c052...v0.3.0

### Added

- controller/db-storage: add initial tariff support. Requires JWT claims to include a `tariff_id`.
- controller: invite verify response contains a `password_required` flag ([#329](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/329))
- controller: add `participant_limit` quota to restrict the maximum amount of participants in a room ([#332](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/332))
- controller: add `enabled_modules` ([#334](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/334)), `tariff` as part of `JoinSuccess` message, API endpoints for `users/me/tariff` and `rooms/{room_id}/tariff` ([#331](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/331))
- controller: add `time_limit` quota to restrict the duration of a meeting ([#333](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/333))
- controller/settings: remove `http.cors` section as CORS is now statically configured to allow any origin
- controller/settings: add `tenants` and `tariffs` sections, which allow configuring how users are assigned to each tenant/tariff.
- legal-vote: add option to set protocol timezone ([#338](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/338))
- janus-media: add `resubscribe` message to allow clients to restart the webrtc session of a subscription.

### Changed

- controller: authenticated users can join meetings without a password ([#335](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/335))
- janus-media: use lapin-pool internally to recover from RabbitMQ connection failures ([#343](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/343))
- lapin-pool: consider connection status when picking connections for new channels & reap disconnected connections ([#343](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/343))
- controller: Traces are now exported directly via OTLP. The setting was renamed from `jaeger_agent_endpoint` to `otlp_tracing_endpoint` ([#301](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/301)).

## [0.2.1] - 2023-03-16

[0.2.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/v0.2.0...v0.2.1

### Added

- `(backport):` janus-media: add `resubscribe` message to allow clients to restart the webrtc session of a subscription.

## [0.2.0] - 2023-03-13

[0.2.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/97c85ca10d136652bc1656792dcf1a539ea4e7a5...v0.2.0

### Added

- controller: enable accepted participants to skip waiting room when joining or returning from a breakout room ([#303](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/303))
- controller: announce available modules in `join_success` message ([#308](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/308))

### Changed

- timer: add the `kind` field to distinguish between a `stopwatch` and `countdown` more clearly ([#316](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/316))
- timer: add the `style` field to the `start` & `started` messages and let clients tag a timer with a custom style ([#316](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/316))
- controller: add support for multi tenancy [#286](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/286)
- timer: distribute timer handling over all participant runners, allowing timers to finish if the moderator has left ([#210](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/210))

## [0.1.0] - 2023-03-01

[0.1.0]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/8b6e62c700376aa82fab9eab07346207becf7c78...v0.1.0

### Added

- add license information
- controller: allow overriding some build-time environment variables ([#137](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/137))
- chat: add `last_seen_timestamp` fields [#242](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/242)
- legal-vote: add option to automatically create a PDF asset when a vote has ended ([#259](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/259))
- legal-vote: add new `live_roll_call` vote kind which sends out live updates while the vote is running ([#285](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/285))
- controller: add config to grant all participants the presenter role by default ([#318](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/318))

### Fixed

- protocol: fixed the `createAuthorIfNotExistsFor` API call that always returned the same author id due to a typo in the query
- janus-media: fixed a permission check for screen-share media session updates
- protocol: fixed a bug where joining participants got write access by default ([#306](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/306))
- protocol: fixed a bug where the etherpad pad was deleted when any user left the room ([#319](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/319))
- signaling: fixed a bug which caused rooms to never be destroyed if a participant was joining from the waiting-room ([#321](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/321))

### Changed

- controller: use derive and attribute macros for conversion to/from redis values ([#283](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/283))
- protocol: read/write access level information is now sent to every participant [#299](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/299)
- chat/ee-chat: merged ee-chat into chat ([#265](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/265))
- legal-vote: votes are now token-based, allowing for `pseudonymous` votings where only the tokens, not the participants are published ([#271](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/271))
- updated dependencies

### Removed

- legal-vote: live updates for `roll_call` and `pseudonymous` votes, results are instead published with the `stopped` message ([#272](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/272))
- automod: will not be part of the community edition ([#257](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/257))
- legal-vote: will not be part of the community edition ([#257](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/257))

## [0.0.0-internal-release.10] - 2022-12-09

[0.0.0-internal-release.10]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/8302382ac420eccc069ca891e0bd067ef6140754...8b6e62c700376aa82fab9eab07346207becf7c78

### Added

- controller: added `waiting_room_state` enum to participants in waiting room ([#245](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/245))

### Fixed

- recording: properly read participants consent when they join or update their state
- recording: only delete the current recording state when the actual recorder leaves

## [0.0.0-internal-release.9] - 2022-12-02

[0.0.0-internal-release.9]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/446647a13f2e163f1be02cefbdaf04e201598444...8302382ac420eccc069ca891e0bd067ef6140754

### Added

- controller: add an S3 storage interface for saving assets in a long-term storage ([#214](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/214))
- whiteboard: save generated PDF files in S3 storage ([#225](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/225))
- legal-vote: add `hidden` parameter to exclude vote choices from outgoing messages ([#260](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/260))
- protocol: save generated PDF files in S3 storage ([#258](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/258))
- controller: add the `recorder` module allowing moderators to record a meeting

### Removed

- controller: `status` field from event resource ([#221](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/221))

### Changed

- controller: introduce `v1/services/..` path for service related endpoints.
- controller: move call-in's start endpoint from `v1/rooms/sip/start` to `v1/services/call_in/start` to make use of the new service authentication.
- controller: trim unnecessary whitespaces in the display name of users and guests ([#96](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/96))

### Fixed

- Respect custom `--version` implementation ([#255](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/255))
- controller: properly handle `is_adhoc` field in the `PATCH events/<event_id>` ([#264](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/264))
- controller: added the missing permission suffix `/assets` when giving access to a room
- controller: fixed a bug where environment variables did not overwrite config values ([#263](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/263))

## [0.0.0-internal-release.8] - 2022-11-10

[0.0.0-internal-release.8]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/8cbc5ed8d23adc95fa3f8e128bbbe84b50977088...446647a13f2e163f1be02cefbdaf04e201598444

### Added

- controller: add `time_independent` filter to events GET request ([#155](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/155))
- mail-worker-protocol: add types to support event-update emails ([#211](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/211))
- controller: send email notification to invitees on event update ([#211](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/211))
- controller: add `suppress_email_notification` flag to event and invite endpoints ([#267](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/267))

### Changed

- strictly follow keep-a-changelog format in `CHANGELOG.md` ([#254](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/254))
- controller: rename `spacedeck` module to `whiteboard` ([#240](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/240))
- controller: return any entry for `GET /events` overlapping `time_min..time_max` range, not just those fully enclosed by it. ([#154](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/154))
- controller: disallow `users/find` queries under 3 characters

### Fixed

- controller/signaling: add missing state checks for control-messages

### Removed

- chat/ee-chat: redundant timestamp removed from outgoing chat messages

## [0.0.0-internal-release.7] - 2022-10-27

[0.0.0-internal-release.7]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/daf5e7e8279bbe48af4240acf74ecbaf8119eb7a...8cbc5ed8d23adc95fa3f8e128bbbe84b50977088

### Added

- controller: added metrics for the number of participants with audio or video unmuted
- controller: add `is_adhoc` flag to events
- chat: allow moderators to clear the global chat history
- janus-media: add the `presenter` role to restrict screenshare access
- janus-media: add reconnect capabilities for mcu clients

### Changed

- controller: runner's websocket error messages straightened (`text` field renamed to `error`, values changed to slug style)

## [0.0.0-internal-release.6] - 2022-10-12

[0.0.0-internal-release.6]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/312226b387dea53679a85c48c095bce769be843b...daf5e7e8279bbe48af4240acf74ecbaf8119eb7a

### Added

- controller: add `waiting_room` flag to event responses

### Fixed

- janus-media: update focus detection on mute

### Changed

- trace: replace the setting `enable_opentelemetry` with `jaeger_agent_endpoint`
- chat/ee-chat: increase maximum chat message size to 4096 bytes

## [0.0.0-internal-release.5] - 2022-09-30

[0.0.0-internal-release.5]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/c3d4da97a1fb32c44956281cc70de6568b3e8045...312226b387dea53679a85c48c095bce769be843b

### Added

- protocol: added the `deselect_writer` action to revoke write access ([#145](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/145))
- controller: added the spacedeck module that allows participants to collaboratively edit a whiteboard ([#209](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/209))
- controller: added a query parameter to the `GET /events` endpoint to allow filtering by `invite_status` ([#213](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/213))
- breakout: added `joined_at` & `left_at` attributes to participants
- controller: toggle raise hands status (actions `enable_raise_hands`, `disable_raise_hands` and according messages) ([#228](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/228))
- controller: added moderator feature to forcefully lower raised hands of all participants ([#227](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/227))
- chat: added feature to toggle chat status (actions `enable_chat`, `disable_chat` and according messages) ([#229](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/229))
- ee-chat: added check for chat status (enabled/disabled) ([#229](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/229))
- controller: added waiting room flag to stored events ([#224](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/224))
- controller: events now include unregistered invitees in invitees lists, distinguishable by `kind` profile property ([#196](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/196))

### Fixed

- controller: fixed a bug where a wrong `ends_at` value for reoccurring events was sent to the mail worker ([#218](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/218))
- controller: fix pagination serialization ([#217](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/217))
- janus-media: added target and type information to some error responses ([#219](https://git.opentalk.dev/opentalk/backend/services/controller/-/issues/219))

## [0.0.0-internal-release.4] - 2022-08-29

[0.0.0-internal-release.4]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/248350563f6de3bd7dab82c2f183a9764fbe68ee...c3d4da97a1fb32c44956281cc70de6568b3e8045

### Added

- controller: added metrics for number of created rooms and number of issued email tasks
- mail-worker-protocol: added `as_kind_str` method to `MailTask`
- controller: added the `timer` module that allows moderators to start a timer for a room
- janus-media: added support for full trickle mode

### Changed

- events-api: added can_edit fields to event related resources
- controller: removed service_name from metrics
- controller: added error context to the keycloak-admin-client
- controller: added the optional claim `nickname` to the login endpoint that will be used as the users `display_name` when set
- janus-media: stopped forwarding RTP media packets while a client is muted

## [0.0.0-internal-release.3] - 2022-07-20

[0.0.0-internal-release.3]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/f001cf6e5a3f7d8e0da29cc9d1c6d1ad744a717f...248350563f6de3bd7dab82c2f183a9764fbe68ee

### Added

- controller: added metrics for number of participants and number of destroyed rooms

### Fixed

- removed static role assignment of participant in breakout and moderation module which led to inconsistent behavior if the participant's role was changed by a moderator
- controller: fixed wrong returned created_by for GET /rooms/{roomId}
- janus-media: added a missing rename for the outgoing error websocket message
- controller: remove special characters from phone numbers before parsing them

### Changed

- updated dependency version of pin-project
- changed the login endpoint to return a bad-request with `invalid_claims` when invalid user claims were provided

## [0.0.0-internal-release.2] - 2022-06-22

[0.0.0-internal-release.2]: https://git.opentalk.dev/opentalk/backend/services/controller/-/compare/b64afd058f6cfa16c67cbbad2f98cd0f2be3181d...f001cf6e5a3f7d8e0da29cc9d1c6d1ad744a717f

### Added

- email-invites: add `ExternalEventInvite` to invite users via an external email address

### Fixed

- config: add `metrics`, `call_in` and `avatar` to settings reload
- controller: set `role` attribute on join
- config: fix room_server.connections example to have better defaults
- controller: respond with 403 instead of 500 when encountering unknown subject in access token
- mail-worker-protocol: fix the `CallIn` and `Room` types to fit their data representation
- janus-client: fixed a race condition where requests were sent before the transaction was registered

### Changed

- update dependency versions of various controller crates

## [0.0.0-internal-release.1] - 2022-06-14

[0.0.0-internal-release.1]: https://git.opentalk.dev/opentalk/backend/services/controller/-/commits/b64afd058f6cfa16c67cbbad2f98cd0f2be3181d

### Added

- initial release candidate

---
