# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.10.0] - 2026-09-10

[0.10.0]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.9.0...v0.10.0

### 🚀 New features

- Include example config in generated signaling module docs ([!1274](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1274))

### 🐛 Bug fixes

- (debriefing) Moderators keep their roles on kick ([!1295](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1295), [#326](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/326))
- (timer) Include ready state when switching breakout rooms ([!1298](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1298), [#327](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/327))
- (meeting_details) Hide meeting details from users ([!1301](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1301), [#329](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/329))
- (meeting_details) Patch show_meeting_details ([!1301](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1301), [#329](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/329))
- (livekit-proxy) Forward websocket pings ([!1306](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1306))

### 📚 Documentation

- Update shared folder module description to include OpenCloud ([!1274](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1274))
- Add breakout room and congested connections metrics docs ([!1304](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1304), [#333](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/333))

### 📦 Dependencies

- (deps) Update pnpm to v11.24.0 ([!1281](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1281))
- (deps) Update rust to 1.98 ([!1282](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1282))
- (deps) Lock file maintenance ([!1282](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1282))
- (deps) Update rust crate yaml-rust2 to 0.12 ([!1269](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1269))
- (deps) Update egui to verion 0.36.1 ([!1246](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1246))
- (deps) Lock file maintenance ([!1302](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1302))
- (deps) Update livekit ([!1308](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1308))
- (deps) Update rust crate dirs to v7 ([!1300](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1300))

### Test

- (livekit-proxy) Ensure user is disconnected when moved to waiting room ([!1297](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1297))

## [0.9.0] - 2026-08-28

[0.9.0]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/opentalk-roomserver-web-api-v0.7.0...v0.9.0

### 🚀 New features

- (moderation) Add an `issued_by` field to `ModerationEvent::RoleUpdated` ([!1254](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1254), [#311](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/311))
- (logging) Add `RoomId` to spans in `ParticipantConnectionTask` ([!1275](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1275))
- (logging) Remove redundant participant ids ([!1275](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1275))
- (room) Include `RoomAlias` in `JoinSuccess` ([!1287](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1287), [#325](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/325))
- Remove the invite code ([!1292](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1292), [#300](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/300))

### 🐛 Bug fixes

- (justfile) Only consider branch local tags when generating changelog ([!1265](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1265))
- (justfile) Use correct version for create-release ([!1267](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1267))
- Leaked waiting room participant when congested ([!1271](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1271), [#314](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/314))
- (chat) Correctly log token bucket size ([!1276](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1276))
- (livekit-proxy) Use GET for rtc/validate endpoint ([!1277](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1277), [#317](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/317))
- Allow concurrent livekit connections ([!1285](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1285), [#320](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/320))
- (orchestrator) Handle resource collision response from orchestrator ([!1292](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1292))

### 🔨 Refactor

- Remove unused `add_conference_connection` method ([!1275](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1275))

### 📦 Dependencies

- (deps) Update pnpm to v11.21.0 ([!1257](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1257))
- (deps) Lock file maintenance ([!1258](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1258))
- (deps) Update grafana/k6 docker tag to v2.2.0 ([!1260](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1260))
- (deps) Update opentalk ([!1264](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1264))
- (deps) Update tungstenite and tokio-tungstenite to v0.30.0 ([!1264](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1264))
- (deps) Update rust crate testcontainers to 0.28.0 ([!1249](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1249))
- (deps) Update pnpm to v11.22.0 ([!1268](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1268))
- (deps) Update h2 to 0.4.16 ([!1270](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1270))
- (deps) Use vendored swagger ui ([!1270](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1270))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.98.0 ([!1288](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1288))
- (deps) Update node.js to v24.20.0 ([!1289](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1289))
- (deps) Update opentalk ([!1292](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1292))
- (deps) Update chacha20 ([!1292](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1292))

### Ci

- (justfile) Use unified versioning ([!1292](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1292))

### Test

- (moderation) Rename misleading test ([!1254](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1254))

## [0.1.4] - 2026-09-09

[0.1.4]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.1.3...v0.1.4

### 🐛 Bug fixes

- (debriefing) Moderators keep their roles on kick ([!1305](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1305), [#326](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/326))
- (recording) Avoid duplicate start requests while pending ([!1310](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1310), [#304](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/304))
- (recorder) Restructure start_recording flow ([!1310](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1310))

### 📦 Dependencies

- (deps) Update chacha20 ([!1311](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1311))

## [0.1.3] - 2026-08-25

[0.1.3]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.1.2...v0.1.3

### 🐛 Bug fixes

- Allow concurrent livekit connections ([!1284](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1284), [#321](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/321))

## [0.1.2] - 2026-08-24

[0.1.2]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.1.1...v0.1.2

### 🐛 Bug fixes

- Leaked waiting room participant when congested ([!1272](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1272), [#315](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/315))
- (livekit-proxy) Use GET for rtc/validate endpoint ([!1278](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1278), [#318](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/318))

### 📦 Dependencies

- (deps) Lock file maintenance ([!1283](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1283))

## [0.1.1] - 2026-08-14

[0.1.1]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/opentalk-roomserver-client-0.1.1...v0.1.1

### 🚀 New features

- (moderation) Add an `issued_by` field to `ModerationEvent::RoleUpdated` ([!1259](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1259), [#312](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/312))

### 🐛 Bug fixes

- (livekit-proxy) Accept participant ids with popout stream suffix ([!1251](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1251))
- (justfile) Only consider branch local tags when generating changelog ([!1266](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1266))

### 📦 Dependencies

- (deps) Update webbrowser to v1.2.4 ([!1266](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1266))

### ⚙ Miscellaneous

- Delete unused file ([!1251](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1251))
- (justfile) Create per-crate releases ([!1266](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1266))
- Use separate version for binary crate ([!1266](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1266))

### Test

- (livekit-module) Ensure connection is alive and can coexist ([!1251](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1251))

## [0.1.0] - 2026-07-07

[0.1.0]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.40...v0.1.0

### 🚀 New features

- Add guest access to `ModeratorJoinInfo` ([!1140](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1140))
- Change environment config prefix to match other OpenTalk services ([!1144](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1144), [#297](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/297))
- (moderation) Send moderator data to new moderators on `ModerationEvent::RoleUpdated` ([!1157](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1157), [#298](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/298))
- (meeting-notes) Log errors when report generation fails ([!1165](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1165))

### 🐛 Bug fixes

- (signaling) Send errors only to the originating connection ([!1130](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1130))
- (core) `CoreEvent::LeftWaitingRoom` only contains a single connection ([!1139](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1139), [#284](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/284))
- Keep signaling websocket open by sending periodic ping frames ([!1147](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1147))
- (room) Participants receive the wrong role in `JoinSuccess` when rejoining after promotion ([!1158](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1158), [#299](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/299))
- (dui) Error due to missing crypto provider ([!1150](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1150))

### ⚡ Performance

- (room) Use read lock in patch_room ([!1168](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1168))

### 🔨 Refactor

- (signaling) Use `is_moderator()` function to check moderator rights ([!1157](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1157))
- (mock) Move recieve event into a function ([!1147](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1147))

### 📦 Dependencies

- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.9 ([!1127](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1127))
- (deps) Update node.js to v24.17.0 ([!1134](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1134))
- (deps) Update pnpm to v11.8.0 ([!1133](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1133))
- (deps) Lock file maintenance ([!1137](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1137), [!1156](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1156), [!1170](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1170))
- (deps) Update pnpm to v11.9.0 ([!1142](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1142))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.26.0 ([!1146](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1146))
- (deps) Update rust crate pdf-extract to 0.12 ([!1152](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1152))
- (deps) Update node.js to v24.18.0 ([!1145](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1145))
- (deps) Update pre-commit hook davidanson/markdownlint-cli2 to v0.23.0 ([!1163](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1163))
- (deps) Update grafana/k6 docker tag to v2.1.0 ([!1161](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1161))
- (deps) Update dependency prettier to v3.9.4 ([!1159](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1159))
- (deps) Update rust crate itertools to 0.15.0 ([!1128](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1128))
- (deps) Update egui ([!1150](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1150))
- (deps) Update rust crate tower-http to 0.7.0 ([!1126](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1126))
- (deps) Update pnpm to v11.10.0 ([!1169](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1169))
- (deps) Document RUSTSEC-2026-0194 and RUSTSEC-2026-0195 ([!1170](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1170))
- (deps) Update crate crossbeam-epoch to v0.9.20 ([!1171](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1171))

### Ci

- Use container build template ([!1149](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1149))
- Use rust-lint template ([!1149](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1149))

### Test

- (moderation) Add (de)serialization tests for moderation state ([!1140](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1140))
- Always use `test-log` with the `trace` feature ([!1165](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1165))

## [0.0.40] - 2026-06-15

[0.0.40]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.39...v0.0.40

### 🚀 New features

- Add transcription module ([!1020](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1020))
- And serialization and integration tests ([!1020](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1020))
- (web-api) Do not require `Clone` for `LiveKitProxyBackend` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (web-api) Abstract livekit websocket adapter in `LiveKitProxyBackend` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (web-api) Expose livekit proxy structs and functions ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (types) Abstract `ContinuationBuffer` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (types) Add actix implementation of `LivekitSocketAdapter` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (types) Implement `From<actix_ws::ProtocolError>` for `websocket::Error` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (types) Implement `From<CloseFrame>` for `actix_ws::CloseReason` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (web-api) Move `axum::Response` out of `LiveKitProxyBackend` trait ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (moderation) Add a `Debrief` variant to the `DisconnectReason` ([!1083](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1083), [#255](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/255))
- (moderation) Prevent guests and call-in users from becoming moderators ([!1089](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1089), [#260](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/260))
- (reaction) Add `reaction` module ([!1109](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1109), [#257](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/257))
- Guest-only waiting room ([!1090](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1090), [#209](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/209))
- Allow fully disabling guest access via `RoomParameters` ([!1090](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1090))

### 🐛 Bug fixes

- (recording) Recording inactive event is sent twice when recording is stopped ([!1087](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1087), [#259](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/259))
- More robust hyphenation ([!1088](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1088), [#258](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/258))
- Word wrapping in all other typst documents ([!1091](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1091))
- (ci) Migrate renovate.json from deprecated `matchPackagePatterns` to `matchPackageNames` ([!1117](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1117), [#275](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/275))
- (test) Client integration tests fail due to missing crypto provider ([!1090](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1090))

### ⚡ Performance

- (web-api) Cache the reqwest client for livekit proxy validate requests ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))

### 📚 Documentation

- (web-api) Add doc comments for `LiveKitProxyBackend` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))

### 🔨 Refactor

- (web-api) Log errors when livekit proxy validate fails ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (test) Use `assert_matches!(value, pattern)` instead of `assert!(matches!(value, pattern))` ([!1108](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1108))
- Replace `matches!()` macro with `matches` in `ClientKind::is_registered_non_callin_user()` ([!1090](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1090))

### 📦 Dependencies

- (deps) Lock file maintenance ([!1081](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1081), [!1111](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1111), [!1122](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1122), [!1097](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1097))
- (deps) Update grafana/k6 docker tag to v2 ([!1074](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1074))
- (deps) Update pre-commit hook embarkstudios/cargo-deny ([!1096](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1096), [!1103](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1103))
- (deps) Update rust crate opentalk-orchestrator-client to 0.8 ([!1093](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1093))
- (deps) Update opentelemetry ([!1069](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1069))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.96.0 ([!1106](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1106))
- (deps) Update livekit ([!1107](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1107))
- (deps) Update node.js to v24.16.0 ([!1095](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1095))
- (deps) Update pnpm ([!1084](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1084), [!1065](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1065), [!1112](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1112), [!1119](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1119), [!1123](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1123))

### ⚙ Miscellaneous

- (justfile) Create gitlab release ([!1115](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1115))

### Ci

- (renovate) Add group rule for opentelemetry ([!1069](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1069))
- Build dev images for release branches ([!1092](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1092), [#244](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/244))

### Test

- (moderation) Add (de)serialization tests for `ModerationError` ([!1089](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1089))
- Add serialization tests send to waiting room command & event ([!1090](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1090))

## [0.0.39] - 2026-05-15

[0.0.39]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.38...v0.0.39

### 🚀 New features

- (web-api) Do not require `Clone` for `LiveKitProxyBackend` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (web-api) Abstract livekit websocket adapter in `LiveKitProxyBackend` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (web-api) Expose livekit proxy structs and functions ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (types) Abstract `ContinuationBuffer` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (types) Add actix implementation of `LivekitSocketAdapter` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (types) Implement `From<actix_ws::ProtocolError>` for `websocket::Error` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (types) Implement `From<CloseFrame>` for `actix_ws::CloseReason` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))
- (web-api) Move `axum::Response` out of `LiveKitProxyBackend` trait ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))

### ⚡ Performance

- (web-api) Cache the reqwest client for livekit proxy validate requests ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))

### 📚 Documentation

- (web-api) Add doc comments for `LiveKitProxyBackend` ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))

### 🔨 Refactor

- (web-api) Log errors when livekit proxy validate fails ([!1078](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1078))

## [0.0.38] - 2026-05-11

### 🚀 New features

- Add transcription module ([!1020](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1020))
- And serialization and integration tests ([!1020](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1020))

## [0.0.37] - 2026-05-11

### 🚀 New features

- (moderation) Add a reason to the `Kicked` moderation event ([!1067](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1067), [#242](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/242))
- (common) Reexport `Internal` settings ([!1075](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1075))

### 🐛 Bug fixes

- (legal-vote) Internal error event when vote is cancelled ([!1068](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1068), [#245](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/245))

### 🔨 Refactor

- (legal-vote) Move timeout cancellation into `ActiveVote` ([!1068](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1068))

### Test

- (moderation) Fix bogus deserialization tests ([!1066](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1066))
- (moderation) Remove type hints from event deserialization tests ([!1066](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1066))

## [0.0.36] - 2026-05-07

### 🚀 New features

- Add own recording consent state to JoinSuccess ([!1047](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1047))
- (room) Configure room idle timeout in the room parameters ([!1041](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1041))
- Create `AssetStorageProvider` and `ModuleResourceProvider` outside the `RoomTask` ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- Move task settings in their own struct ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- (room) Move shared resources of the room task into a new `RoomTaskContext` struct ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- Move ban check into `RoomTaskHandle` ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- Reexport types needed in the controller ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- (common) Derive `PartialEq` and `Eq` for settings structs used in the controller ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- (types) Add actix feature ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- (signaling) Allow creating internal storage errors without anyhow ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- (signaling) Allow creating module resource errors without anyhow ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))

### 🐛 Bug fixes

- (ci) Build resolute image instead of noble ([!1053](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1053))
- Do not alter livekit query when proxying requests ([!1050](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1050), [#241](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/241))
- Whisper token subjects need to contain the connection id ([!1050](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1050))
- Remove left over cors ([!1060](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1060))
- (livekit-proxy) Allow tls connections to livekit ([!1062](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1062))

### 🔨 Refactor

- Use generic SignalingSocket types ([!1009](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1009))
- Make opentalk-roomserver-room independent from axum ([!1009](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1009), [#224](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/224))
- Use URL type in Livekit settings ([!1050](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1050))
- Simplify accessing used quota from `TariffDetails` ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))
- Simplify `RoomParameters::calc_time_limit_quota` ([!1052](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1052))

### 📦 Dependencies

- (deps) Lock file maintenance ([!1051](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1051))
- (deps) Update ubuntu docker tag to v26 ([!1039](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1039))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.25.0 ([!1054](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1054))
- (deps) Update rust crate utoipa to v5.5.0 ([!1058](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1058))
- (deps) Update rust crate sysinfo to 0.39 ([!1063](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1063))

### ⚙ Miscellaneous

- Update README ([!1049](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1049))
- (just) Allow to only plot specific crates ([!1009](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1009))

## [0.0.35] - 2026-04-27

[0.0.35]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.34...v0.0.35

### 🚀 New features

- Move livekit proxy out of v1 scope ([!1034](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1034))
- Log unknown fields ([!1037](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1037))
- Add transcription client kind ([!1038](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1038))

### 🐛 Bug fixes

- (excalidraw) Publish excalidraw module crate ([!1029](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1029))
- (md) Formatting ([!1040](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1040))

### 📦 Dependencies

- (deps) Update pnpm to v10.33.1 ([!1027](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1027))
- (deps) Update orchestrator client ([!1030](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1030))
- (deps) Insta and pretty_assertions are only used in tests ([!1035](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1035))
- (deps) Update pnpm to v10.33.2 ([!1036](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1036))
- (deps) Update opentalk ([!1044](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1044))

### Ci

- Use lint template ([!1040](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1040))

## [0.0.34] - 2026-04-22

[0.0.34]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.33...v0.0.34

### 🚀 New features

- Add 404 fallback handler ([!1007](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1007))
- Expose module ids in `opentalk-roomserver-modules` ([!1014](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1014))
- Move roomserver specific types out of opentalk-types-api-v1 ([!997](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/997))
- (chat) Include chat rate limit delays in `SlowDown` and `TooManyRequests` ([!1002](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1002), [#226](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/226))
- (excalidraw) Introduce excalidraw whiteboard ([!905](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/905), [#192](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/192))
- Reexport `ERROR_MODULE_ID` ([!1025](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1025))

### 🐛 Bug fixes

- (chat) Rate limit sends slow_down too early ([!988](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/988))
- (moderation) Call-in users are not kicked by the `Guests` kick scope ([!994](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/994))
- Convert environment variables to their expected types ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- Load test script should not print colored logs into a file ([!1012](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1012))
- Fix typos ([!1013](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1013))
- (chat) Enforce non-zero values for token bucket size and tokens per second in chat rate limit ([!1002](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1002))
- `ParticipantsFiltered` does not apply filters in `contains()` ([!905](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/905))
- (openapi) Add missing status for `v1/signaling/{token}` ([!905](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/905))
- (automod) Speaker list can include participants from other breakout rooms ([!1017](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1017))
- (legal-vote) Break long words in technical issue description ([!1023](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1023), [#233](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/233))
- (load) Use correct environment variables in 'spike storage quota' ([!1018](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1018))
- Only call crypto provider once ([!1018](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1018))
- `SetParameter` API request returned NotImplemented ([!1018](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1018))
- Fairness test used 'seconds' postfix but actually used ms ([!1018](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1018))
- (OpenAPI) Add missing paths and fix wrong types ([!1018](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1018))
- Recording should be configured using the `RoomParameters` ([!1024](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1024))
- (client) Add missing `SignalingModuleEvent::Error` variant ([!1025](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1025), [#232](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/232))

### 🔨 Refactor

- Rename `MAX_INFLIGHT` to `TOTAL_SEND` ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- (api) Make `TokenStore` generic over its value ([!905](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/905))
- Move `TokenStore` to the `opentalk-roomserver-common` crate ([!905](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/905))
- (api) Make cors layer available to all apis ([!905](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/905))
- Add `From` source error implementation for `AssetLoadError` ([!905](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/905))
- Rename error namespace constant to match current conventions ([!1025](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1025))
- Use ModuleId from roomserver crate ([!1025](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1025))

### 📦 Dependencies

- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.2 ([!996](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/996))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.4 ([!1001](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1001))
- (deps) Update pnpm to v10.33.0 ([!1004](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1004))
- (deps) Update dependency prettier to v3.8.3 ([!999](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/999))
- (deps) Update node.js to v24.14.1 ([!1003](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1003))
- (deps) Use `rustls-tls-native-roots` with livekit crate everywhere ([!1010](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1010))
- (deps) Update rust crate rustls-webpki 0.103 to 0.103.12 ([!1010](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1010))
- (deps) Document RUSTSEC-2026-0098 & RUSTSEC-2026-0099 ([!1010](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1010))
- (deps) Update node.js to v24.15.0 ([!1011](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1011))
- (deps) Update rust to 1.95.0 ([!1021](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1021))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.95.0 ([!1022](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1022))
- (deps) Update rust crate tokio to v1.52.1 ([!998](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/998))
- (deps) Lock file maintenance ([!1016](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1016))
- (deps) Document RUSTSEC-2026-0104 ([!1026](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1026))

### ⚙ Miscellaneous

- Remove livekit settings (dead code) ([!1018](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1018))

### Ci

- Switch ci image to alpine base ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- Ensure load-test formatting ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- Add scripts to test load tests ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- Test load test scripts ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- (renovate) Only match dependencies starting with opentalk in the opentalk group ([!1021](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/1021))

### Test

- (load) Add environment configuration to fairness test ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- (load) Add environment configuration to spike-join load test ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- (load) Add env configuration to spike-storage-quota load test ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))
- (load) Don't abort when test ends ([!980](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/980))

## [0.0.33] - 2026-04-13

[0.0.33]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.32...v0.0.33

### 🚀 New features

- (livekit) Proxy livekit requests ([!938](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/938), [#199](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/199))

### 🔨 Refactor

- Move SecurityAddon to web-api crate ([!938](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/938))

### 🐛 Bug fixes

- (livekit) Error log when participant disconnects without sending close frame ([!978](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/978))
- (moderation) Do not allow kicked users to skip the waiting room ([!981](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/981))
- (moderation) Restrict kicking moderators to the room owner ([!981](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/981))

### 📦 Dependencies

- (deps) Update egui ([!972](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/972))
- (deps) Update opentalk ([!973](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/973), [!992](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/992))
- (deps) Update tungstenite to 0.29.0 ([!977](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/977))
- (deps) Update rust crate opentalk-orchestrator-client to 0.5 ([!979](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/979))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.1 ([!984](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/984), [!985](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/985))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.1 ([!986](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/986))
- (deps) Lock file maintenance ([!991](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/991))

### Ci

- (renovate) Add group rule for tungstenite ([!977](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/977))
- Publish container images to harbor ([!982](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/982))

## [0.0.32] - 2026-04-07

[0.0.32]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.31...v0.0.32

### 🚀 New features

- Provide available features inside JoinSuccess ([!965](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/965), [#211](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/211))

### 🐛 Bug fixes

- Add missing utoipa feature ([!960](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/960))
- (load) Update room parameters in load tests to match the roomserver ([!963](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/963))
- Allow `GET` and `CONNECT` methods to upgrade ws requests ([!964](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/964))
- Enable TLS for outgoing requests (i.e. LiveKit api) ([!964](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/964))

### 🔨 Refactor

- (load) Allow configuring user ID when connecting client ([!963](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/963))

### 📦 Dependencies

- (deps) Update opentalk rust crates ([!967](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/967))

### ⚙ Miscellaneous

- (logs) Log livekit error details ([!964](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/964))
- Create dependency graph for all opentalk dependencies ([!966](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/966))

### Ci

- Use changelog preview template ([!962](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/962))

### Test

- (load) Add storage quota spike test ([!963](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/963))

## [0.0.31] - 2026-03-31

[0.0.31]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.30...v0.0.31

### 🐛 Bug fixes

- Add missing utoipa feature ([!960](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/960))

## [0.0.30] - 2026-03-31

[0.0.30]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.29...v0.0.30

### 🚀 New features

- (client) Derive `Clone` for `Client` ([!935](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/935))
- (web-api) Add REST endpoint for updating storage quota ([!935](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/935), [#200](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/200))

### 🐛 Bug fixes

- Make ModuleRegistry::print output deterministic ([!958](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/958), [#213](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/213))

## [0.0.29] - 2026-03-27

[0.0.29]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.28...v0.0.29

### 🚀 New features

- Add core features ([!956](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/956), [#207](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/207))

### 🐛 Bug fixes

- Ensure CryptoProvider is configured ([!951](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/951))
- Room parameter update do not include empty password ([!952](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/952), [#208](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/208))

### 🔨 Refactor

- Add crate to setup CryptoProviders ([!953](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/953))

### 📦 Dependencies

- (deps) Lock file maintenance ([!953](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/953), [!955](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/955))
- (deps) Update crate unicode-segmentation to v1.13.2

### Ci

- (pre-commit) Add machete to pre-commit hooks ([!953](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/953))

## [0.0.28] - 2026-03-17

[0.0.28]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.27...v0.0.28

### 🐛 Bug fixes

- (room) Compiler error in room metrics file when used as a dependency ([!949](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/949), [#206](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/206))

## [0.0.27] - 2026-03-17

[0.0.27]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.26...v0.0.27

### 📦 Dependencies

- (deps) Update orchestrator client to 0.4.1 ([!947](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/947))

### ⚙ Miscellaneous

- Do not use default-feature for service auth crate ([!946](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/946))

## [0.0.26] - 2026-03-16

[0.0.26]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.25...v0.0.26

### 🚀 New features

- Print SignalingModule descriptions ([!941](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/941), [#172](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/172))
- Introduce `opentalk-roomserver-modules` crate ([!941](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/941))
- (room) Expose a map from `ModuleId` to their provided `FeatureId` ([!941](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/941))
- (orchestrator) Allow service registration by port ([!936](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/936))

### 🔨 Refactor

- Avoid circular dependency and prepare release ([!942](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/942), [#202](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/202))
- (settings) Don't assume a default service url ([!936](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/936))
- This isn't a mock, it's the actual thing ([!943](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/943))

### 📦 Dependencies

- (deps) Update orchestrator client ([!936](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/936))
- (deps) Update `opentalk-types-api-v1` to v0.55.0 ([!944](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/944))

### ⚙ Miscellaneous

- (recorder) Use `tracing` crate instead of `log` ([!940](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/940))

### Ci

- (cargo) Don't publish recording module crate ([!940](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/940))

## [0.0.25] - 2026-03-11

[0.0.25]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.24...v0.0.25

### 🚀 New features

- Add recording module ([!517](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/517), [#137](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/137))

### 🔨 Refactor

- Add RoomKind to ClientKind::Recorder ([!933](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/933))
- Replace streaming_links with streaming_targets in client parameters ([!517](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/517))

### ⚙ Miscellaneous

- (justfile) Rename `update-frontend-api` function to `update-openapi` ([!937](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/937))
- Add documentation for ClientKind::Recorder's room field ([!517](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/517))

## [0.0.24] - 2026-03-05

[0.0.24]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.23...v0.0.24

### 🚀 New features

- Add controller module resource storage ([!913](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/913))
- (mock) Add recorder participant ([!921](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/921))
- Add `show_meeting_details` field to `RoomParameters` ([!930](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/930))
- (web-api) Patch room parameters ([!930](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/930), [#198](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/198))

### 🐛 Bug fixes

- (ci) Gitlab token not passed to git-cliff when defined in file ([!903](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/903))
- (openapi) Remove obsolete `TariffModuleResource` ([!915](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/915))
- `service_url` fails to build when `http.address` is an IPv6 address ([!904](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/904), [#188](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/188))
- (whiteboard) Initialization state is not reset when initialization fails ([!917](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/917))
- (meeting-notes) Initialization state is not reset when initialization fails ([!916](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/916))
- (ci) Reduce usage of nightly compiler for running tests ([!925](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/925), [#197](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/197))
- Service participants do not skip waiting room ([!921](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/921), [#195](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/195))
- (core) Core modules are not in the `enabled_modules` of the `JoinSuccess` message ([!922](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/922), [#196](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/196))

### 📚 Documentation

- Add `http.service_url` ([!904](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/904))

### 🔨 Refactor

- Use internal controller api ([!914](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/914))
- Use total and used instead of remaining quota ([!931](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/931))
- (room) Rename `Request::UpdateParameter` to `SetParameters` ([!930](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/930))

### 📦 Dependencies

- (deps) Lock file maintenance ([!923](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/923), [!908](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/908), [!927](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/927))
- (deps) Update opentalk ([!920](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/920))
- (deps) Update rust crate sysinfo to v0.38.3 ([!928](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/928))

## [0.0.23] - 2026-02-13

[0.0.23]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.22...v0.0.23

### 🚀 New features

- Add controller settings ([!891](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/891))
- (ci) Add release mr creation job ([!902](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/902), [#191](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/191))
- (room) Restrict CORS based on allowed origins in `RoomParameters` ([!879](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/879), [#167](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/167))

### 📚 Documentation

- (room) Add description to `ws_rate_limit` field in `RoomParameters` ([!879](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/879))

### 🔨 Refactor

- Remove AssetStorageConfig from RoomParameters ([!894](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/894))

### 📦 Dependencies

- (deps) Update rust crate toml to v1 ([!898](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/898))

## [0.0.22] - 2026-02-10

[0.0.22]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.21...v0.0.22

### 🔨 Refactor

- (room) Use `ModuleSettings` to determine which modules to initialize ([!875](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/875))

### 📦 Dependencies

- (deps) Update rust crate rand to 0.10 ([!886](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/886))

## [0.0.21] - 2026-02-09

[0.0.21]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.20...v0.0.21

### 🚀 New features

- (types) Add default implementation for `RateLimitSettings` ([!884](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/884))

### 🔨 Refactor

- (types) Move `RateLimitSettings` to a separate module ([!884](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/884))

### 📦 Dependencies

- (deps) Lock file maintenance ([!887](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/887))
- (deps) Update livekit ([!888](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/888))

## [0.0.20] - 2026-02-06

[0.0.20]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.19...v0.0.20

### 🚀 New features

- (metrics) Track the number of disconnects due to congestion ([!850](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/850), [#180](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/180))
- (room) Implement websocket rate limiting ([!864](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/864), [#185](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/185))
- Add service url to http config ([!855](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/855))
- Add orchestrator client ([!855](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/855))

### 🐛 Bug fixes

- (load) Log correct message when token parsing fails as error ([!845](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/845))
- (load) Load tests log `undefined` on websocket errors ([!852](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/852))
- (ci) Scan `dev-$FLAVOR` tags instead of the non existing `latest-$FLAVOR` ([!860](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/860))
- (client) Avoid leading slashes when joining url paths ([!861](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/861))
- (chat) `SetLastSeenTimestamp` is sent to all participants ([!866](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/866), [#187](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/187))
- Increase event buffer size ([!864](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/864), [#185](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/185))

### ⚡ Performance

- (e2ee) Remove the e2ee disconnect event ([!858](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/858))
- (load) Optimize client memory usage ([!853](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/853))
- (load) Reduce load test setup memory usage ([!853](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/853))

### 🔨 Refactor

- (metrics) Make breakout counter functions associated ([!850](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/850))
- (load) Add JSDoc comments to functions ([!845](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/845))
- (load) Add load testing client ([!845](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/845))
- Make SignalingConnection::close take self by reference instead of owned ([!871](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/871))
- (chat) Remove deprecated group scope ([!868](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/868))

### 📦 Dependencies

- (deps) Update dependency prettier to v3.8.1 ([!849](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/849))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.93.0 ([!851](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/851))
- (deps) Lock file maintenance ([!854](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/854), [!863](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/863))
- (deps) Update rust crate opentalk-etherpad-client to 0.5 ([!856](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/856))
- (deps) Update rust crate bytes to v1.11.1 ([!869](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/869))
- (deps) Update opentalk ([!872](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/872))
- (deps) Remove @types/k6 dependency ([!874](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/874))
- (deps) Update service-auth & orchestrator-client ([!877](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/877))
- (deps) Update rust crate testcontainers to 0.27.0 ([!881](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/881))
- (deps) Update time crate ([!883](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/883))

### ⚙ Miscellaneous

- (advisory) Remove unencountered RUSTSEC-2025-0134 ([!883](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/883))

### Test

- (load) Remove echo-stress load tests ([!845](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/845))
- (load) Add fairness load test ([!845](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/845), [#180](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/180))
- (load) Add join spike test ([!852](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/852), [#180](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/180))
- (load) Add join and chat break tests ([!853](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/853), [#180](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/180))

## [0.0.19] - 2026-01-21

[0.0.19]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.18...v0.0.19

### 🚀 New features

- (metrics) Add signaling metrics ([!765](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/765), [#170](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/170))
- Limit max number of breakout rooms ([!816](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/816))
- Support binding to ipv4 and ipv6 concurrently ([!813](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/813), [#177](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/177))
- Remove default for `public_url` config ([!815](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/815))
- Disable OpenAPI/Swagger endpoints by default ([!826](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/826), [#181](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/181))
- Allow guests to change their own display name ([!837](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/837), [#183](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/183))
- Add health command ([!844](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/844), [#159](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/159))

### 🐛 Bug fixes

- `JoinedWaitingRoom` core event only contains the last `ConnectionId` ([!776](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/776))
- (mock) Compiler error when compiling tests without mock feature ([!765](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/765))
- Ensure a close frame is send after a close frame was received ([!833](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/833))
- Ensure commands from disconnected participants don't reach RoomTask ([!834](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/834))
- Allow to overwrite array using environment variables ([!843](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/843))

### ⚡ Performance

- Send websocket messages in batches ([!833](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/833))

### 📚 Documentation

- Add observability docs ([!768](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/768))
- Add overview section ([!811](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/811), [#164](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/164))
- Add conference docs ([!822](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/822), [#164](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/164))
- Rename 'defaults and fallback values' to defaults in the navigation ([!821](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/821), [#164](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/164))
- Add link to reports docs in configuration ([!820](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/820), [#164](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/164))
- Add http server docs ([!810](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/810), [#164](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/164))

### 🔨 Refactor

- (moderation) Simplify `From` implmentation for `WaitingParticipantPeerData` ([!777](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/777))
- (metrics) Move `build_prometheus_layer` function to metrics module ([!765](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/765))
- Derive `Copy` and `Hash` for `ParticipationKind` ([!765](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/765))
- Implement `From<&ClientKind>` for `ParticipationKind` ([!765](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/765))
- (automod) Rename `StateMachineOutput` to `SpeakerSelectionOutput` ([!809](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/809))
- Remove `u64` from frontend API ([!816](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/816), [#163](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/163))
- (http) Determine default address when parsing config ([!839](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/839))

### 📦 Dependencies

- (deps) Lock file maintenance ([!779](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/779), [!825](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/825))
- (deps) Lock file maintenance ([!841](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/841))
- (deps) Upate crate `opentalk-version` to v0.4.0 ([!841](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/841))
- (deps) Update dependency linguify to v0.5.0 ([!789](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/789))
- (deps) Update egui to v0.33.3 ([!767](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/767))
- (deps) Update livekit ([!832](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/832))
- (deps) Update livekit/livekit-server docker tag to v1.9.10 ([!760](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/760))
- (deps) Update opentalk ([!780](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/780), [!842](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/842))
- (deps) Update pre-commit hook adrienverge/yamllint to v1.38.0 ([!829](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/829))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.24.0 ([!830](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/830))
- (deps) Update pre-commit hook andrejorsula/pre-commit-cargo to v0.5.0 ([!797](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/797))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.21.7 ([!790](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/790))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.0 ([!818](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/818))
- (deps) Update registry.opencode.de/opentalk/spacedeck docker tag to v3 ([!788](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/788))
- (deps) Update rust crate axum to v0.8.8 ([!791](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/791))
- (deps) Update rust crate axum-prometheus to 0.10.0 ([!806](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/806))
- (deps) Update rust crate cidr to v0.3.2 ([!784](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/784))
- (deps) Update rust crate derive_more to v2.1.1 ([!792](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/792))
- (deps) Update rust crate insta to v1.45.0 ([!786](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/786))
- (deps) Update rust crate reqwest to 0.13 ([!802](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/802))
- (deps) Update rust crate serde_json to v1.0.147 ([!796](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/796))
- (deps) Update rust crate testcontainers to v0.26.2 ([!787](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/787))
- (deps) Update rust crate toml to v0.9.10 ([!783](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/783))
- (deps) Update rust crate yaml-rust2 to 0.11 ([!782](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/782))
- (deps) Update typst to v0.14.2 ([!778](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/778))

### ⚙ Miscellaneous

- Ignore vacuum rules ([!803](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/803))
- Enable default features for reqwest ([!802](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/802))
- Remove unnecessary argument ([!834](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/834))

### Ci

- Ignore the dui for code coverage ([!819](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/819))
- Use testcontainer instead of gitlab services ([!838](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/838))
- (clippy) Replace custom script with `self_named_module_files` lint ([!697](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/697))

### Test

- (waiting-room) Test `ConnectionIds` in `ModeratorJoinInfo` ([!776](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/776))
- (legal-vote) Add serialization test for token vote voting record ([!816](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/816))
- Remove external services from tests ([!838](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/838))
- (load) Add echo stress load test ([!817](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/817), [#179](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/179), [#180](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/180))
- (load) Add load test monitoring tools ([!817](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/817), [#178](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/178))
- (load) Add docker compose for roomserver ([!817](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/817))

## [0.0.18] - 2025-12-12

[0.0.18]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.17...v0.0.18

### 🚀 New features

- (metrics) Restrict access to the metrics endpoint by ip address ([!764](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/764), [#169](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/169))

### 🐛 Bug fixes

- (ci) Add default job condition for `cargo_test` ([!773](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/773))
- (ci) Don't run tests on main ([!774](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/774))

### 📚 Documentation

- Add defaults docs ([!771](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/771))
- Add meeting report docs ([!772](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/772))
- Add cli docs ([!769](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/769))

### 📦 Dependencies

- (deps) Update pre-commit hook daveshanley/vacuum to v0.20.5 ([!766](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/766))
- (deps) Update rust crate tower-http to v0.6.8 ([!759](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/759))
- (deps) Update rust crate reqwest to v0.12.25 ([!761](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/761))
- (deps) Update opentalk-types ([!770](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/770))

## [0.0.17] - 2025-12-09

[0.0.17]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.16...v0.0.17

### 🚀 New features

- Add report creation date to attendance reports ([!742](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/742))
- (l10n) Generate pdf reports in the language of the room owner ([!742](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/742))

### 🐛 Bug fixes

- (legal-vote) Typo in function name ([!742](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/742))

### 📦 Dependencies

- (deps) Update livekit/livekit-server docker tag to v1.9.7 ([!752](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/752))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.9 ([!758](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/758))

### Test

- (legal-vote) Remove duplicate test ([!742](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/742))

## [0.0.16] - 2025-12-08

[0.0.16]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.15...v0.0.16

### 🚀 New features

- (api) Use opentalk-service-auth for authorization ([!725](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/725))
- (moderation) Include connection ids of waiting participants in moderator join info ([!751](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/751), [#171](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/171))
- (docs) Prepare documentation for mkdocs-material ([!753](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/753))

### 🐛 Bug fixes

- (ci) Assume that no CVEs should be ignored when .trivyignore-$FLAVOR does not exist ([!710](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/710))
- (ci) `read-trivy-ignore-file` fails when no trivy ignore file exist ([!741](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/741))
- Update the api key section in the example config ([!743](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/743))

### 📦 Dependencies

- (deps) Lock file maintenance ([!726](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/726))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.20.2 ([!714](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/714))
- (deps) Lock file maintenance ([!737](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/737))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.6 ([!727](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/727))
- (deps) Update pre-commit hook markdownlint/markdownlint to v0.15.0 ([!730](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/730))
- (deps) Update rust crate testcontainers to 0.26.0 ([!738](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/738))
- (deps) Update livekit/livekit-server docker tag to v1.9.6 ([!712](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/712))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.20.4 ([!739](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/739))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.7 ([!740](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/740))
- (deps) Update rust crate log to v0.4.29 ([!746](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/746))
- (deps) Update rust crate derive_more to v2.1.0 ([!745](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/745))
- (deps) Update rust crate opentalk-types-api-v1 to 0.48.0 ([!744](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/744))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.8 ([!747](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/747))
- (deps) Update typst to v0.14.1 ([!748](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/748))
- (deps) Update alpine docker tag to v3.23 ([!749](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/749))
- (deps) Update rust crate opentalk-types-common to v0.40.1 ([!750](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/750))

### Ci

- Host pages for main branch ([!754](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/754))
- Pages could not be build ([!755](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/755))

## [0.0.15] - 2025-11-21

[0.0.15]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.14...v0.0.15

### 🐛 Bug fixes

- (training-participation-report) Publish training participation report types ([!704](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/704))

### 🔨 Refactor

- Use `opentalk-report-generation` instead of a custom crate ([!701](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/701))

### 📦 Dependencies

- (deps) Lock file maintenance ([!713](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/713))
- (deps) Update opentalk-types ([!723](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/723))

### Ci

- Add from scratch image ([!706](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/706))

## [0.0.14] - 2025-11-12

[0.0.14]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.13...v0.0.14

### 🚀 New features

- Add legal vote signaling module ([!632](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/632), [#74](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/74))
- Accept waiting participants when disabling the waiting room ([!700](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/700), [#162](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/162))
- (training-participation-report) Add training-participantion-report signaling module ([!645](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/645), [#81](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/81))

### 🐛 Bug fixes

- (echo) Remove unnecessary `PeerJoinInfo` and logs ([!695](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/695))
- (ci) `lint-mod-file` wrongfully complains about directories ending in 'rs' ([!632](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/632))

### 🔨 Refactor

- (room) Expose user id in `ModuleContext` ([!632](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/632))
- (mock) Rename `TestRoom::stored_file()` to `stored_asset()` ([!645](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/645))

### 📦 Dependencies

- (deps) Update rust crate opentalk-types-common to v0.39.1 ([!696](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/696))
- (deps) Lock file maintenance ([!699](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/699))
- (deps) Update rust crate service-probe to 0.3.0 ([!698](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/698))

### Test

- (livekit) Fix slow livekit tests ([!694](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/694))
- (core) Add test for patching the root element of a module resource ([!632](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/632))
- (mock) Expose module resource storage for tests ([!632](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/632))
- (mock) Allow retrieving all files from a `TestRoom` ([!645](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/645))

## [0.0.13] - 2025-11-06

[0.0.13]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.12...v0.0.13

### 🚀 New features

- Allow for the room idle timeout to be configurable ([!680](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/680))
- (dui) Add moderator tools plugin ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))
- Add controller asset storage ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))

### 🐛 Bug fixes

- Remove rooms from the registry when they are done ([!680](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/680))

### 🔨 Refactor

- (test) Rename all `serde_tests` modules to `tests` ([!678](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/678))
- Replace `Duration::from_secs` with `from_mins`/`from_hours` where it makes sense ([!683](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/683))
- Join blocked event to include reason field ([!690](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/690))
- (shared-folder) Remove event error variant ([!691](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/691))
- (signaling) Keep consistent wording in asset API ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))
- Replace chunk uploading with stream implementation ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))
- Rename `StorageError::QuotaReached` to `QuotaExceeded` ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))

### 📦 Dependencies

- (deps) Update rust crate livekit to v0.7.24 ([!658](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/658))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.2 ([!674](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/674))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.3 ([!676](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/676))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.4 ([!679](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/679))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.91.0 ([!682](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/682))
- (deps) Update livekit/livekit-server docker tag to v1.9.3 ([!685](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/685))
- (deps) Lock file maintenance ([!687](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/687))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.5 ([!689](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/689))
- (deps) Update opentalk ([!692](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/692))

### ⚙ Miscellaneous

- (shared-folder) Adhere to module directory naming convention ([!691](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/691))
- (client) Add additional tracing spans to improve error tracking ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))
- (meeting-report) Log errors before loosing context ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))

### Ci

- Add doctests ([!681](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/681))

### Test

- (polls) Migrate polls serialization tests to insta ([!684](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/684))
- (polls) Add missing (de)serialization tests ([!684](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/684))
- Cover ControllerAssetStorage ([!617](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/617))

## [0.0.12] - 2025-10-29

[0.0.12]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.11...v0.0.12

### 📦 Dependencies

- (deps) Update pre-commit hook fsfe/reuse-tool to v6.2.0 ([!669](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/669))
- (deps) Update opentalk ([!670](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/670))
- (deps) Remove unnecessary `vergen` and `vergen-gix` dependencies ([!672](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/672))

### ⚙ Miscellaneous

- Set `opentalk-roomserver` as default member ([!673](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/673))

## [0.0.11] - 2025-10-27

[0.0.11]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.10...v0.0.11

### 🚀 New features

- Attach the participants breakout room to the `ParticipantJoined` event data ([!634](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/634), [#149](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/149))
- Enforce room participant limit quota ([!636](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/636), [#146](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/146))
- (chat) Message rate limit ([!578](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/578))
- (room) Graceful room shutdown ([!650](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/650), [#150](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/150))
- (api) Require version specifier for API endpoints ([!663](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/663), [#157](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/157))

### 🐛 Bug fixes

- (just) `mktemp --suffix` doesn't work on MacOS ([!623](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/623))
- (chat) Include breakout chat history on join ([!635](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/635), [#151](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/151))
- (core) Tag `CoreEvent` and enforce snake_case for serialization ([!637](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/637))
- (docs) Wrong status code when request body could not be parsed for PUT `/room/{room_id}/token` ([!656](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/656))
- (client) Remove unnecessary `tokio` and `url` dev dependencies ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))
- (api) Include status code in reply when room parameters are missing ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))
- (docs) Document all possible errors for put_room and request_token ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))

### 🔨 Refactor

- (report-generation) Remove obsolete error variants ([!625](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/625))
- (report-generation) Use `thiserror` ([!625](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/625))
- (polls) Move `Poll` struct from types to module ([!641](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/641))
- Loosen trait bound on `ModuleContext::loopback_after()` ([!641](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/641))
- (timer) Simplify filtering participants for room ([!641](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/641))
- Expose the RoomBackend for the orchestrator ([!648](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/648), [#154](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/154))
- (web-api) Return HTTP 422 for PUT `/room/{room_id}/token` when `RoomParameters` are missing ([!656](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/656), [#136](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/136))
- (client) Remove `http-request-derive` and improve error handling ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))
- (meeting-notes) Use loopback task for deleting pads ([!650](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/650))

### 📦 Dependencies

- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.23.0 ([!624](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/624))
- (deps) Lock file maintenance ([!628](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/628))
- (deps) Update rust crate pdf-extract to 0.10 ([!626](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/626))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6 ([!630](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/630))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6.1.2 ([!631](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/631))
- (deps) Update opentelemetry crates ([!622](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/622))
- (deps) Lock file maintenance ([!639](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/639))
- (deps) Update egui ([!638](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/638))
- (deps) Update livekit ([!640](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/640))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.6 ([!647](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/647))
- (deps) Update rust crate half to 2.7.1 ([!641](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/641))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.7 ([!649](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/649))
- (deps) Update livekit/livekit-server docker tag to v1.9.2 ([!651](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/651))
- (deps) Lock file maintenance ([!652](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/652))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.8 ([!653](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/653))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.9 ([!657](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/657))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.0 ([!659](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/659))
- (deps) Update opentalk ([!654](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/654))
- (deps) Update typst rust crates ([!665](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/665))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.19.1 ([!666](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/666))
- (deps) Lock file maintenance ([!667](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/667))

### Ci

- Do not include a default config in the containers ([!629](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/629))
- (renovate) Add group rule for typst ([!665](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/665))

### Test

- (mock) Increase socket timeout to 3 seconds ([!633](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/633))
- (chat) Add serialization tests for breakout messages ([!635](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/635))
- (core) Unify `CoreEvent` serialization tests ([!637](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/637))
- (client) Add integration tests for client crate ([!660](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/660))
- (meeting-notes) Pin etherpad container for local tests to v2.0.2 ([!650](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/650))

## [0.0.10] - 2025-09-30

[0.0.10]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.9...v0.0.10

### 🚀 New features

- (core) Add module resources interface ([!556](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/556))

### 🔨 Refactor

- Organize storage module locations ([!556](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/556))

### 📦 Dependencies

- (deps) Update rust crate opentalk-etherpad-client to 0.4 ([!616](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/616))
- (deps) Update opentalk ([!618](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/618))

### ⚙ Miscellaneous

- Switch to internal kaniko image ([!587](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/587))

## [0.0.9] - 2025-09-29

[0.0.9]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.8...v0.0.9

### 🚀 New features

- (types) Add retain function to `ModuleSettings` ([!613](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/613))

### 🐛 Bug fixes

- (room) Use `ParticipantState.display_name` in `ParticipantJoined` if participant already exists ([!604](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/604))
- (room) `CoreCommand::EnterRoom` while already in room returns `SignalingError::UnknownNamespace` ([!612](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/612), [#147](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/147))

### 🔨 Refactor

- (room) Lower trace level for unsupported core commands ([!612](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/612))

### ⚙ Miscellaneous

- (justfile) Update frontend api version number with `set-version` ([!614](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/614))

## [0.0.8] - 2025-09-29

[0.0.8]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.7...v0.0.8

### 🚀 New features

- (whiteboard) Add whiteboard signaling module ([!569](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/569), [#82](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/82))
- Include public url in token response ([!595](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/595), [#145](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/145))
- (roomserver) Per-connection command streams to enable websocket rate limiting ([!586](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/586))
- Allow to configure the asset storage via RoomParameter ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))
- (siganling) Add context information to `StorageProvider` ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))

### 🐛 Bug fixes

- (client) `SignalingModuleEvent::namespace()` returns the wrong id for subroom audio ([!576](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/576))
- Pass correct participant id for core peer data ([!594](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/594))
- (room) Send `JoinedWaitingRoom` event when a participant is moved to the waiting room ([!602](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/602))
- (room) Participants' `in_waiting_room` state is not reset when joining the conference ([!603](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/603))

### 📚 Documentation

- (echo) Document echo types ([!600](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/600))

### 🔨 Refactor

- Apply `clippy::pedantic` lints ([!575](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/575))
- Allow signaling modules to store files when being destroyed ([!569](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/569))
- Replace `ModuleData` with `ModuleSettings` in `RoomParameters` ([!585](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/585), [#144](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/144))
- (automod) Inline structs in event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (chat) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (chat) Rename chat module error to `ChatError` ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (e2ee) Inline structs in command ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (meeting-report) Inline structs in event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (moderation) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (polls) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (raise-hands) Inline structs in command ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (subroom-audio) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (timer) Inline structs in command & event ([!593](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/593))
- (echo) Remove event error variant ([!600](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/600))
- (echo) Reexport command and event ([!600](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/600))
- (room) Separate conference and waiting room commands early ([!589](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/589))
- Add `Asset` to the asset storage related type names ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))
- (mock) Replace `wait_accept` function with `enter_room` and use it in tests ([!603](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/603))

### 📦 Dependencies

- (deps) Lock file maintenance ([!581](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/581))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.90.0 ([!582](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/582))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.5 ([!584](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/584))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.5 ([!583](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/583))
- (deps) Update rust crate config to v0.15.17 ([!591](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/591))
- (deps) Update rust crates tungstenite and tokio-tungstenite to 0.28.0 ([!590](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/590))
- (deps) Update rust crate serde to v1.0.227 ([!596](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/596))
- (deps) Lock file maintenance ([!608](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/608))
- (deps) Update rust crate livekit to v0.7.19 ([!609](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/609))

### ⚙ Miscellaneous

- Remove old scripts that are replaced by the DUI ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))
- Remove old values from the example config ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))

### Ci

- Use fixed container tags ([!599](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/599))
- (renovate) Add group rule for livekit ([!609](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/609))

### Test

- (whiteboard) Add integration tests for whiteboard module ([!569](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/569))
- (mock) Don't allow to overwrite RoomParameters in RoomBuilder ([!598](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/598))

## [0.0.7] - 2025-09-19

[0.0.7]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.6...v0.0.7

### 🐛 Bug fixes

- (types) Remove unnecessary dependency ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))
- Remove self referencing dependency ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))

### 🔨 Refactor

- (test) Move tests to moderation module ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))

### 📦 Dependencies

- (deps) Update pre-commit hook daveshanley/vacuum to v0.18.3 ([!572](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/572))
- (deps) Update rust crate config to v0.15.16 ([!565](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/565))
- (deps) Update rust crate serde to v1.0.224 ([!568](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/568))
- (deps) Update rust crate serde to v1.0.225 ([!573](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/573))
- (deps) Update rust crate serde_path_to_error to v0.1.20 ([!566](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/566))

### ⚙ Miscellaneous

- (logging) Optional loopbacks are no errors ([!548](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/548))
- Use glob pattern for modules ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))
- Mark unpublished crates with `publish = false` ([!571](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/571))

### Ci

- Run livekit test in ci ([!548](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/548), [#88](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/88))

### Test

- Optionally use provided livekit server ([!548](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/548))
- (client) Add serialization tests for meeting notes command & event ([!570](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/570))

## [0.0.6] - 2025-09-15

[0.0.6]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.5...v0.0.6

### 🚀 New features

- (chat) Implement server side message search ([!469](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/469))
- (moderation) Change display name ([!476](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/476))
- (client) Add core command to client library ([!486](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/486))
- (types) Add `from_u128` impl to `DeviceId` ([!489](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/489))
- (moderation) Implement enable/disable waiting room ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (moderation) Implement sent to waiting room ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (moderation) Send waiting room `JoinInfo` to moderators ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (raise-hands) Raise hands ([!485](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/485))
- (dui) Add timer plugin ([!503](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/503))
- (signaling) Add participant state to JoinInfo ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- Allow SignalingModules to add participant data ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- (timer) Add ready_state of peers to join success ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- Log if a module was skipped during initialization ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- (livekit) Add force mute internal command ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- (automod) Automod ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- Add call-in client kind ([!514](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/514))
- (room) Include module data about participants when switching breakout rooms ([!504](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/504), [#134](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/134))
- Rename peer data related fields ([!504](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/504))
- (subroom-audio) Add SubroomAudio Module ([!510](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/510))
- (dui) Add more default clients parameter ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Enable all modules in default room parameter ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) About popup ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Report more error details while connecting to roomserver ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Use roomserver api token as default ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Use all available editor width ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Add event details and shared folder information to room parameter ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (dui) Allow to suspend and resume websocket receiving ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- (dui) Add SpamAmountPlugin for spamming ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- (room) Send ParticipantDisconnected event when connection is dropped ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- (moderation) Add ban/unban commands ([!509](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/509))
- (room) Add timestamp to `SignalingEvent` ([!527](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/527))
- (room) Room time limit ([!530](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/530))
- (moderation) Allow moderators to change participant roles ([!537](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/537))
- (room) Add signaling event for commands sent from the waiting room ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (ping) Respond to pings from the waiting room ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (types) Introduce participation kind ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (core) Add peer event data when participant joins ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519), [#118](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/118))
- (raise-hands) Use PeerEvent and PeerData instead of custom map ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (core) Include participant information in JoinedWaitingRoom event ([!550](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/550))
- (storage) Add functions to upload a file in chunks ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))
- (meeting-notes) Meeting notes ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))
- (moderation) Notify moderators about accepted participants ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))
- (dui) Waiting room plugin ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))
- (dui) Use transaction IDs in plugins ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))

### 🐛 Bug fixes

- (chat) Prevent sending private messages to unknown participants ([!470](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/470))
- Tag core command for serialization ([!486](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/486))
- Prevent participants in waiting room from receiving broadcast events ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (client) Use `payload` instead of `content` ([!503](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/503))
- Don't flatten Participant::module_data ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- Don't error if directory already exists ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- Send events and module messages in the correct order ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- (livekit) Send the correct identifier when muting a participant ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- Register raised hands signaling module ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (room) Don't sleep 1 sec if connection becomes unresponsive ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513), [#132](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/132))
- (livekit) Ensure muting works across breakout rooms ([!518](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/518))
- (moderation) Only notify participants that were unmuted when muting ([!518](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/518))
- (moderation) Enable waiting room when kicking participant ([!520](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/520))
- (moderation) Prevent room owner from getting kicked ([!520](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/520))
- (moderation) Regular users can enable/disable the waiting room ([!528](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/528))
- (dui) Widgets have duplicate ids when the same message is sent multiple times ([!529](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/529))
- (room) Tests do not compile when testing single module ([!530](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/530))
- (room) Waiting participants don't receive errors for core commands ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (room) Core and breakout module features are missing in `JoinSuccess` ([!536](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/536))
- (livekit) Moderators can't screenshare when default screen share permission is false ([!543](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/543))
- (timer) Remove unnecessary serde tag ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (raise-hands) Only reset raised hand if last connection is closed ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (ci) Add trivy ignore file for trixie ([!551](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/551))
- (ci) Use nightly cargo for `cargo-fmt` in pre-commit hook ([!558](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/558))
- (dui) Don't dive into a busy loop when disconnecting ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))

### 📚 Documentation

- Fix mixed up field descriptions for `ModuleJoinData` and `ModuleSwitchData` ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))

### 🔨 Refactor

- (breakout) Move breakout module types to roomserver-types crate ([!489](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/489))
- (room) Deduplicate event serialization ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (meeting-report) Remove `ParticipantKind` from `ReportParticipant` ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (room) Store `ClientKind` and `Role` in `WaitingParticipant` ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- Use `ClientKind` in `ParticipantState` ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- Don't use RawJson ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- Remove async from functions where not necessary ([!515](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/515))
- (livekit) Rename `ForceMute` command to `Mute` ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- Remove result callback from internal command ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- Move microphone restriction commands to moderation module ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- Move mute command to moderation module ([!512](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/512))
- (room) Move core functions to core ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- Remove unnecessary mutex ([!513](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/513))
- (moderation) Add missing error variant doc comments ([!520](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/520))
- (justfile) Determine current version in functions ([!532](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/532))
- (room) Replace `IdleTimeout` with a generic `Timeout` ([!530](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/530))
- (room) Simplify `RoomTask::execute_signaling_module_command` ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (room) Do not attempt to send error messages when no connection exist ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (room) Ensure disconnected participants can't sent commands ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- (echo) Remove obsolete code from echo module ([!525](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/525))
- Introduce participant_id_from_uuid function ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- Add breakout join data methods ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- Don't panic on unserializable type ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (livekit) Remove livekit client from `LivekitConnection` ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))
- Replace `FsStorage` with `MemoryFileStorage` ([!557](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/557))
- Implement `Copy` for `DisconnectReason` ([!531](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/531))

### 📦 Dependencies

- (deps) Lock file maintenance ([!472](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/472), [!493](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/493), [!506](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/506), [!524](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/524), [!545](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/545), [!564](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/564))
- (deps) Update egui to v0.32.3 ([!555](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/555))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.89.0 ([!468](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/468))
- (deps) Update opentalk ([!497](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/497))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.13 ([!559](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/559))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.4 ([!487](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/487))
- (deps) Update pre-commit hook fsfe/reuse-tool to v5.1.1 ([!544](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/544))
- (deps) Update pre-commit hook pre-commit/pre-commit-hooks to v6 ([!471](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/471))
- (deps) Update rust crate anyhow to v1.0.99 ([!479](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/479))
- (deps) Update rust crate async-trait to v0.1.89 ([!488](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/488))
- (deps) Update rust crate axum-prometheus to 0.9.0 ([!484](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/484))
- (deps) Update rust crate chrono to v0.4.42 ([!546](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/546))
- (deps) Update rust crate clap to v4.5.47 ([!533](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/533))
- (deps) Update rust crate config to v0.15.14 ([!482](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/482))
- (deps) Update rust crate insta to v1.43.2 ([!542](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/542))
- (deps) Update rust crate livekit to v0.7.18 ([!534](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/534))
- (deps) Update rust crate livekit-api to v0.4.6 ([!535](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/535))
- (deps) Update rust crate log to v0.4.28 ([!539](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/539))
- (deps) Update rust crate opentalk-types-api-v1 to v0.40.1 ([!511](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/511))
- (deps) Update rust crate opentalk-types-common to v0.36.1 ([!483](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/483))
- (deps) Update rust crate reqwest to v0.12.23 ([!480](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/480))
- (deps) Update rust crate serde_json to v1.0.143 ([!496](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/496))
- (deps) Update rust crate thiserror to v2.0.16 ([!498](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/498))
- (deps) Update rust crate url to v2.5.6 ([!502](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/502))

### ⚙ Miscellaneous

- Register missing modules ([!473](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/473))
- Adapt implementation for web compatibility ([!422](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/422))
- Rename ping module to echo for naming compatibility ([!495](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/495))
- Remove unused echo commands ([!495](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/495))
- Add insta files to git ignore ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- Wrap comments with rustfmt ([!538](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/538))
- (justfile) Check if yq is installed before running `tag-release` action ([!552](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/552))

### Ci

- Update debian based image trixie ([!475](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/475))
- Update debian based ci images to trixie ([!501](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/501))
- (nextest) Run all tests by default ([!553](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/553))
- Use nextest and llvm-cov for test coverage and report ([!553](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/553))
- Run tests on main ([!553](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/553))
- (just) Add scripts to get test coverage ([!553](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/553))

### Test

- (chat) Add test for breakout room history access permissions ([!469](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/469))
- (chat) Error when sending private messages to unknown participants ([!470](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/470))
- (types) Test serialization of `Participant` ([!489](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/489))
- Add tests that ensure waiting participants don't receive events ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (moderation) Add integration tests for enable/disable waiting room ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (moderation) Add integrations tests for send to waiting room ([!474](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/474))
- (raise-hands) Add integrations tests for raise hands ([!485](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/485))
- (timer) Introduce `start_timer` helper function ([!494](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/494))
- (types) Switch to insta ([!507](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/507))
- Move common livekit functions to separate crate ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- (automod) Add integration tests for automod module ([!490](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/490))
- (room) Add join info and switch into to MockModule ([!504](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/504))
- Integration test for switch and start breakout rooms ([!504](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/504))
- Use uniform test user names ([!526](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/526))
- (moderation) Verify livekit behavior when testing mute ([!518](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/518))
- (mock) Participants orderly disconnect in integration tests ([!547](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/547))
- Don't include connection_id as it's random and complicates testing ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (mock) Verify that participant switched room ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- (raise-hands) Ensure state is reset when changing rooms ([!519](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/519))
- Allow configuring `receive_event` timeout ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))
- (meeting-notes) Add meeting notes tests ([!549](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/549))

## [0.0.5] - 2025-08-07

[0.0.5]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.4...v0.0.5

### 🐛 Bug fixes

- Add missing crate features ([!466](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/466))

## [0.0.4] - 2025-08-07

[0.0.4]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.3...v0.0.4

### 🚀 New features

- (mock) Use explicit RoomParameters for test ([!426](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/426))
- (mock) Better errors when events are not sent ([!426](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/426))
- (mock) Add module init data ([!426](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/426))
- (dui) Store and migrate versioned settings ([!427](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/427))
- (mock) Allow to set the Event in the RoomBuilder ([!390](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/390))
- (shared-folder) Add SharedFolder SignalingModule ([!390](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/390))
- (client) Add SharedFolder event and command types ([!390](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/390))
- Add meeting start, end and timezone to `EventContext` ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Add joined/left timestamps to participant state ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Add email to participant state ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Add `visible()` filter for participant state ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Add report generation crate ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (meeting-report) Add meeting-report types ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (meeting-report) Add meeting-report module ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (test) Configure logging to also support tracing logs ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (room) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (chat) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (ping) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (polls) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (shared-folder) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (roomserver) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (web-api) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (signaling) Replace `log` with `tracing` ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- Inter module communication ([!441](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/441))
- (room) Instroduce waiting room ([!412](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/412), [#114](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/114))
- (room) Add core commands ([!450](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/450))
- (moderation) Implement kick participants instruction ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (moderation) Add kick and debrief types ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (moderation) Implement kick and debrief in moderation module ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (client) Add moderation to client library ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (chat) Chunk message history ([!411](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/411))

### 🐛 Bug fixes

- (release) Add descriptions for polls and timer types crates ([!424](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/424))
- (justfile) Recommend correct yq variant when prompting to install ([!423](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/423))
- (dui) Fix layout to show parameter save button ([!427](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/427))
- Shut down modules when exiting room task ([!438](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/438))
- (ci) Add trivy ignore file for alpine ([!439](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/439))
- (ci) Initialize module data in livekit tests ([!446](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/446))
- (room) Close websocket when disconnecting participant ([!450](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/450))
- (room) Second close frame when disconnecting participant ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))

### 🔨 Refactor

- Simplify `ParticipantKind` enum ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (storage) Include filename in `AssetUploaded` ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (storage) Allow using the storage api in an async context ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (room) Move command handling to function ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (room) Introduce functions for joining the room ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- (room) Introduce function to execute SignalingModule commands ([!440](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/440))
- Make `build_join_success` an associated function of `RoomTask` ([!441](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/441))
- (signaling) Allow better chaining of filter and iterator ([!412](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/412))
- Simplify comments in message router ([!450](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/450))
- (room) Rename `result_handle` field/parameter to `result_callback` ([!450](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/450))
- (room) Remove `ModulePeerData` from `JoinedWaitingRoom` event ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (signaling) Allow better chaining of participant id filter ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Rename core commands to instructions ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Make entering from the waiting room a core command/event ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (moderation) Move accept to moderation module ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Improve tracing when performing close handshake ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Make `MessageRouter::remove_connection()` synchronous ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))
- (room) Move timezone to `PublicUserProfile` and make it required ([!456](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/456))

### 📦 Dependencies

- (deps) Update rust crate egui_json_tree to v0.12.1 ([!429](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/429))
- (deps) Update rust crate livekit to v0.7.15 ([!428](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/428))
- (deps) Lock file maintenance ([!434](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/434))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.6 ([!430](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/430))
- (deps) Update rust crate egui_json_tree to 0.13.0 ([!435](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/435))
- (deps) Update `opentalk-types-common` to 0.35.3 ([!436](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/436))
- (deps) Lock file maintenance ([!445](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/445))
- (deps) Update rust crate testcontainers to 0.25.0 ([!444](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/444))
- (deps) Downgrade rust crate rustls to 0.23.29 ([!447](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/447))
- (deps) Update rust crate clap to v4.5.42 ([!449](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/449))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.7 ([!454](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/454))
- (deps) Update rust crate serde_json to v1.0.142 ([!453](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/453))
- (deps) Update rust crate livekit to v0.7.16 ([!451](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/451))
- (deps) Lock file maintenance ([!458](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/458))
- (deps) Update rust crate ecow to v0.2.6 ([!459](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/459))
- (deps) Update rust crate opentalk-types-common to v0.35.4 ([!460](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/460))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.8 ([!461](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/461))
- (deps) Update rust crate opentalk-types-common to v0.35.5 ([!462](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/462))
- (deps) Update rust crate clap to v4.5.43 ([!463](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/463))
- (deps) Update opentalk ([!464](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/464))

### ⚙ Miscellaneous

- Add description to all crates ([!426](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/426))
- (test) Require using the result of received_nothing ([!411](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/411))

### Ci

- (justfile) Make all recipes quiet ([!425](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/425))
- (justfile) Remove shebang from recipes when unnecessary ([!425](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/425))
- (justfile) Tag release ([!425](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/425))
- Update packages of alpine image on each build ([!442](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/442))

### Test

- Allow accessing stored files from `FsStorage` ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- Allow configuring storage quota ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (meeting-report) Add integration tests for meeting-report module ([!431](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/431))
- (moderation) Add integration tests for kick and debrief ([!448](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/448))

## [0.0.3] - 2025-07-15

[0.0.3]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.2...v0.0.3

### 🚀 New features

- (client) Add timer types ([!394](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/394))
- (client) Add polls types ([!396](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/396))
- Configure modules with room parameter ([!402](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/402))
- Storage API interface ([!389](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/389))
- (signaling) Filter for moderators ([!413](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/413))

### 🔨 Refactor

- (chat) Remove peer_state directory ([!397](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/397))

### 📦 Dependencies

- (deps) Lock file maintenance ([!401](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/401))
- (deps) Update rust crate config to v0.15.12 ([!405](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/405))
- (deps) Update rust crate clap to v4.5.41 ([!407](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/407))
- (deps) Update rust crate config to v0.15.13 ([!406](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/406))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.3 ([!416](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/416))
- (deps) Update pre-commit hook daveshanley/vacuum to v0.17.5 ([!418](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/418))
- (deps) Lock file maintenance ([!415](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/415))
- (deps) Update pre-commit hook pre-commit/pre-commit-hooks to v3.4.0 ([!419](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/419))
- (deps) Update pre-commit hook adrienverge/yamllint to v1.37.1 ([!417](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/417))
- (deps) Update pre-commit hook pre-commit/pre-commit-hooks to v5 ([!420](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/420))
- (deps) Update rust crate egui to 0.32.0 ([!409](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/409))

### ⚙ Miscellaneous

- Remove opentalk-types-signaling-chat dependency ([!397](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/397))

### Ci

- (renovate) Add egui group ([!409](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/409))

## [0.0.2] - 2025-07-04

[0.0.2]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/compare/v0.0.1...v0.0.2

### 🚀 New features

- (signaling) Add default implementation for `on_loopback_event` ([!377](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/377))
- (e2ee) Introduce e2ee signaling module ([!377](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/377))
- (client) Add e2ee types ([!377](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/377))
- (ping) Remove test commands from ping ([!384](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/384), [#95](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/95))
- (chat) Remove serde feature from chat types crate ([!385](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/385), [#86](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/86))
- (roomserver-client) Remove feature flags to avoid openssl dependency ([!398](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/398))

### 🔨 Refactor

- Break cycle and move test to room crate ([!387](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/387), [#105](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/issues/105))
- Move timer types into roomserver ([!393](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/393))

### 📦 Dependencies

- (deps) Update rust crate http-request-derive-client-reqwest to 0.2.0 ([!391](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/391))
- (deps) Update rust crate reqwest to v0.12.22 ([!386](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/386))
- (deps) Update rust crate tokio to v1.46.0 ([!388](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/388))

### Ci

- Add alpine based image and make it the default ([!392](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/392))

### Test

- (mock) Introduce Dave to the gang ([!377](https://git.opentalk.dev/opentalk/backend/services/roomserver/-/merge_requests/377))

## [0.0.1] - 2025-07-01

[0.0.1]: https://git.opentalk.dev/opentalk/backend/services/roomserver/-/tags/v0.0.1

The initial `RoomServer` release!

This release contains the following crates:

- `opentalk-roomserver-types`
- `opentalk-roomserver-signaling`
- `opentalk-roomserver-client`
- `opentalk-roomserver-signaling`
- `opentalk-roomserver-types`
- `opentalk-roomserver-types-chat`
- `opentalk-roomserver-types-livekit`
- `opentalk-roomserver-types-ping`
