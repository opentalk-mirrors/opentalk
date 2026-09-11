# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.19.0] - 2026-09-10

[0.19.0]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.18.0...v0.19.0

### 🚀 New features

- Render room alias instead of ID when set ([!805](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/805), [#188](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/188))
- Replace invite link with join link, remove `invite_code` ([!772](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/772), [#185](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/185))

### 🐛 Bug fixes

- (justfile) Fetch version correctly when commiting release ([!807](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/807))
- (justfile) Only consider branch local tags when generating changelog ([!807](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/807))
- Log phone number parsing failures with debug level and remove leading whitespace ([!820](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/820), [#189](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/189))

### 📚 Documentation

- Migration guide 0.17 -> 0.18 ([!827](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/827))
- Add migration 0.18 -> 0.19 ([!827](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/827))
- Fix changelog for 0.17.0 and 0.18.0 ([!827](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/827))

### 🔨 Refactor

- Simplify `format_telephone_number_filter` error handling ([!820](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/820))

### 📦 Dependencies

- (deps) Lock file maintenance ([!811](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/811), [!819](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/819), [!823](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/823))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust ([!816](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/816), [!817](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/817))
- (deps) Update rust crate rstest to 0.27.0 ([!822](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/822))
- (deps) Update rust crate tera-contrib to 0.3.0 ([!814](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/814))

## [0.18.0] - 2026-08-14

[0.18.0]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.17.0...v0.18.0

### 🚀 New features

- (docs) Add developer documentation for profiling memory usage ([!341](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/341))
- Warn about unknown configuration fields ([!724](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/724))
- Change the environment prefix from `MAILER` to `OPENTALK_MAIL` ([!762](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/762))

### 🐛 Bug fixes

- Use correct queue name in example config ([!724](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/724))
- (ci) Generated docs are not updated ([!768](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/768))
- (docs) Remove outdated development section ([!768](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/768))
- (rabbitmq) Use durable non-exclusive queue ([!778](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/778), [#184](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/184))
- (justfile) Fetch version correctly when commiting release ([!808](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/808))
- (justfile) Only consider branch local tags when generating changelog ([!808](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/808))

### 📚 Documentation

- Update outdated rabbitmq docs ([!768](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/768))

### 📦 Dependencies

- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.0 ([!707](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/707))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.1 ([!713](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/713))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.1 ([!715](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/715))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.94.1 ([!714](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/714))
- (deps) Lock file maintenance ([!716](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/716))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.2 ([!718](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/718))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.4 ([!720](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/720))
- (deps) Lock file maintenance ([!717](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/717))
- (deps) Lock file maintenance ([!727](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/727))
- (deps) Update opentalk-controller to 0.44.0 ([!614](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/614))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.25.0 ([!728](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/728))
- (deps) Lock file maintenance ([!729](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/729))
- (deps) Lock file maintenance ([!732](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/732))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.5 ([!731](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/731))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.6 ([!733](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/733))
- (deps) Update rust crate fluent-templates to 0.14.0 ([!726](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/726))
- (deps) Lock file maintenance ([!736](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/736))
- (deps) Lock file maintenance ([!739](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/739))
- (deps) Update rust crate log to v0.4.30 ([!740](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/740))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.7 ([!738](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/738))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.8 ([!741](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/741))
- (deps) Update rust crate opentalk-types-common to v0.44.1 ([!742](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/742))
- (deps) Lock file maintenance ([!746](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/746))
- (deps) Update rust crate log to v0.4.31 ([!747](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/747))
- (deps) Lock file maintenance ([!750](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/750))
- (deps) Lock file maintenance ([!753](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/753))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.9 ([!754](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/754))
- (deps) Lock file maintenance ([!759](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/759))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.26.0 ([!761](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/761))
- (deps) Lock file maintenance ([!767](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/767))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.96.0 ([!723](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/723))
- (deps) Update ubuntu docker tag to v26 ([!725](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/725))
- (deps) Lock file maintenance ([!771](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/771))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.97.0 ([!773](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/773))
- (deps) Lock file maintenance ([!777](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/777))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.20.2 ([!774](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/774))
- (deps) Update pre-commit hook davidanson/markdownlint-cli2 to v0.23.1 ([!769](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/769))
- (deps) Lock file maintenance ([!786](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/786))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.97.1 ([!788](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/788))
- (deps) Update rust crate css-inline to 0.21.0 ([!755](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/755))
- (deps) Update opentalk-controller to 0.46.0 ([!781](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/781))
- (deps) Update pre-commit hook davidanson/markdownlint-cli2 to v0.23.2 ([!794](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/794))
- (deps) Lock file maintenance ([!793](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/793))
- (deps) Update rust crate itertools to 0.15 ([!756](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/756))
- (deps) Update rust crate serial_test to v4 ([!792](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/792))
- (deps) Update rust crate tera to v2 ([!765](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/765))
- (deps) Lock file maintenance ([!798](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/798))
- (deps) Lock file maintenance ([!803](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/803))

### ⚙ Miscellaneous

- Fix clippy lints ([!798](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/798))

### Ci

- (just) Switch changelog tool to opentalk git-cliff ([!700](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/700))
- Use changelog template ([!702](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/702))
- (pre-commit) Use same markdownlint from CI in pre-commit ([!717](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/717))
- (pre-commit) Switch from taplo to olpat (taplo is unmaintained) ([!775](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/775))

## [0.17.0] - 2026-03-10

[0.17.0]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.16.0...v0.17.0

### 🚀 New features

- (ci) Add release mr creation job ([!678](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/678), [#175](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/175))
- (ci) Switch to buildah and use ci templates ([!692](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/692))
- Use fluent-langneg for language negotiation ([!697](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/697))

### 🐛 Bug fixes

- (ci) Conditions for read-tags CI job ([!641](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/641))
- (log) Print mail sending errors as ERROR instead of WARN ([!675](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/675), [#174](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/174))

### 🔨 Refactor

- Update opentalk-types crates ([!697](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/697))

### 📦 Dependencies

- (deps) Update pre-commit hook andrejorsula/pre-commit-cargo to v0.5.0 ([!644](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/644))
- (deps) Update rust crate serde_json to v1.0.147 ([!643](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/643))
- (deps) Update rust crate url to v2.5.8 ([!652](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/652))
- (deps) Mark RUSTSEC-2025-0141 as accepted ([!657](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/657))
- (deps) Update rust crate clap to v4.5.54 ([!649](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/649))
- (deps) Update rust crate serde_json to v1.0.149 ([!645](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/645))
- (deps) Update pre-commit hook adrienverge/yamllint to v1.38.0 ([!654](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/654))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.24.0 ([!655](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/655))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.19.0 ([!653](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/653))
- (deps) Update rust crate insta to v1.46.0 ([!647](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/647))
- (deps) Update rust crate serial_test to v3.3.1 ([!651](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/651))
- (deps) Update rust crate tokio to v1.49.0 ([!650](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/650))
- (deps) Update rust crate css-inline to 0.19.0 ([!648](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/648))
- (deps) Update rust crate chrono to v0.4.43 ([!658](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/658))
- (deps) Update rust crate insta to v1.46.1 ([!659](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/659))
- (deps) Update rust crate phonenumber to v0.3.9 ([!656](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/656))
- (deps) Update rust crate service-probe-client to 0.4.0 ([!662](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/662))
- (deps) Update rust crate opentalk-version to 0.4.0 ([!660](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/660))
- (deps) Update rust crate service-probe to 0.4.0 ([!661](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/661))
- (deps) Lock file maintenance ([!646](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/646))
- (deps) Update rust crate uuid to v1.20.0 ([!665](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/665))
- (deps) Update rust crate css-inline to v0.19.1 ([!664](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/664))
- (deps) Lock file maintenance ([!666](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/666))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.93.0 ([!663](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/663))
- (deps) Lock file maintenance ([!669](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/669))
- (deps) Lock file maintenance ([!670](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/670))
- (deps) Lock file maintenance ([!674](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/674))
- (deps) Update rust crate clap to v4.5.58 ([!676](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/676))
- (deps) Update rust crate css-inline to 0.20.0 ([!672](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/672))
- (deps) Lock file maintenance ([!683](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/683))
- (deps) Update rust crate clap to v4.5.59 ([!685](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/685))
- (deps) Update rust crate fluent-templates to v0.13.3 ([!686](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/686))
- (deps) Update rust crate clap to v4.5.60 ([!687](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/687))
- (deps) Lock file maintenance ([!690](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/690))
- (deps) Lock file maintenance ([!694](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/694))
- (deps) Update rust crate tokio to v1.50.0 ([!696](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/696))
- (deps) Update rust crate snafu to 0.9.0 ([!695](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/695))
- (deps) Lockfile maintenance ([!695](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/695))

### ⚙ Miscellaneous

- (container) Remove debian bookworm image ([!698](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/698), [#178](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/178))

## [0.16.0] - 2025-11-13

[0.16.0]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.15.2...v0.16.0

### 🚀 New features

- Add from scratch container build ([!589](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/589))
- Health command ([!605](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/605), [#159](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/159))

### 🐛 Bug fixes

- (rabbitmq) Make handling of rabbitmq tasks more robust ([!533](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/533), [#149](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/149))
- (ci) Allow-list CVE-2025-6297 for Bookworm ([!541](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/541), [#148](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/148))

### 🔨 Refactor

- (logging) Set log level of verbose crates separately ([!576](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/576))

### 📦 Dependencies

- (deps) Update rust crate percent-encoding to v2.3.2 ([!524](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/524))
- (deps) Update rust crate url to v2.5.6 ([!526](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/526))
- (deps) Update rust crate tokio-executor-trait to v2.2.0 ([!529](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/529))
- (deps) Lock file maintenance ([!532](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/532))
- (deps) Update rust crate fluent-templates to v0.13.1 ([!535](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/535))
- (deps) Update rust crate clap to v4.5.46 ([!536](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/536))
- (deps) Update rust crate snafu to v0.8.8 ([!537](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/537))
- (deps) Lock file maintenance ([!539](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/539))
- (deps) Update rust crate uuid to v1.18.1 ([!540](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/540))
- (deps) Update rust crate clap to v4.5.47 ([!542](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/542))
- (deps) Update rust crate log to v0.4.28 ([!544](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/544))
- (deps) Update rust crate snafu to v0.8.9 ([!543](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/543))
- (deps) Update rust crate insta to v1.43.2 ([!546](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/546))
- (deps) Update pre-commit hook fsfe/reuse-tool to v5.1.0 ([!545](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/545))
- (deps) Update pre-commit hook fsfe/reuse-tool to v5.1.1 ([!547](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/547))
- (deps) Lock file maintenance ([!548](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/548))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.5 ([!558](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/558))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.90.0 ([!556](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/556))
- (deps) Update git.opentalk.dev:5050/opentalk/tools/opentalk-ci-doc-updater docker tag to v0.2.0 ([!528](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/528))
- (deps) Update rust crate config to v0.15.17 ([!559](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/559))
- (deps) Update rust crate serde to v1.0.227 ([!560](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/560))
- (deps) Lock file maintenance ([!562](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/562))
- (deps) Update pre-commit hook alessandrojcm/commitlint-pre-commit-hook to v9.23.0 ([!565](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/565))
- (deps) Lock file maintenance ([!566](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/566))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6 ([!567](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/567))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6.1.2 ([!568](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/568))
- (deps) Update rust crate lettre to v0.11.19 ([!569](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/569))
- (deps) Lock file maintenance ([!571](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/571))
- (deps) Update rust crate clap to v4.5.49 ([!572](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/572))
- (deps) Lock file maintenance ([!580](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/580))
- (deps) Update opentalk-controller ([!563](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/563))
- (deps) Update rust crate clap to v4.5.50 ([!581](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/581))
- (deps) Update rust crate opentalk-types-common to v0.37.1 ([!583](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/583))
- (deps) Lock file maintenance ([!590](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/590))
- (deps) Update pre-commit hook fsfe/reuse-tool to v6.2.0 ([!591](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/591))
- (deps) Update rust crate clap to v4.5.51 ([!594](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/594))
- (deps) Update rust crate opentalk-version to 0.3.0 ([!600](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/600))
- (deps) Lock file maintenance ([!599](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/599))
- (deps) Lock file maintenance ([!607](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/607))
- (deps) Update rust crate service-probe to 0.3.0 ([!606](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/606))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.91.0 ([!604](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/604))
- (deps) Update rust crate css-inline to 0.18.0 ([!598](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/598))

### ⚙ Miscellaneous

- Update default ci and container image to Debian Trixie ([!527](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/527), [#146](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/146))
- (code) Update rust edition to 2024 ([!534](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/534))
- Switch to internal kaniko image ([!557](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/557))
- (deny) Allowlist unmaintained unic-* crates until our dependency has migrated ([!587](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/587))

## [0.15.0] - 2025-08-21

[0.15.0]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.14.0...v0.15.0

### 🚀 New features

- Print license information ([!438](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/438))

### 📦 Dependencies

- (deps) Lock file maintenance ([!463](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/463), [!469](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/469), [!473](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/473), [!477](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/477), [!479](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/479), [!482](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/482), [!486](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/486), [!491](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/491), [!497](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/497), [!502](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/502), [!510](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/510), [!519](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/519))
- (deps) Update alpine docker tag to v3.22 ([!462](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/462))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.89.0 ([!507](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/507), [!478](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/478))
- (deps) Update pre-commit hook adrienverge/yamllint to v1.37.1 ([!488](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/488))
- (deps) Update pre-commit hook embarkstudios/cargo-deny to v0.18.4 ([!517](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/517), [!487](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/487))
- (deps) Update pre-commit hook pre-commit/pre-commit-hooks to v6 ([!509](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/509))
- (deps) Update rust crate anyhow to v1.0.99 ([!513](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/513))
- (deps) Update rust crate clap to v4.5.45 ([!514](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/514), [!512](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/512), [!505](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/505), [!499](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/499), [!485](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/485), [!470](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/470))
- (deps) Update rust crate config to v0.15.14 ([!515](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/515), [!484](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/484), [!483](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/483))
- (deps) Update rust crate css-inline to 0.17.0 ([!496](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/496), [!489](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/489), [!476](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/476))
- (deps) Update rust crate lapin to v2.5.4 ([!493](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/493))
- (deps) Update rust crate lettre to v0.11.18 ([!498](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/498))
- (deps) Update opentalk-types rust crates ([!506](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/506), [!492](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/492), [!503](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/503), [!504](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/504))
- (deps) Update rust crate rstest to 0.26.0 ([!495](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/495))
- (deps) Update rust crate serde_json to v1.0.143 ([!520](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/520), [!500](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/500))
- (deps) Update rust crate snafu to v0.8.7 ([!521](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/521))
- (deps) Update rust crate tokio to v1.46.0 ([!480](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/480))
- (deps) Update rust crate uuid to v1.18.0 ([!511](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/511))

### ⚙ Miscellaneous

- Move back to single crate structure ([!464](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/464))
- Add cargo-machete to ci and pre-commit ([!466](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/466))
- (justfile) Include the update-changelog job in prepare-release ([!522](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/522))

## [0.14.0] - 2025-05-29

[0.14.0]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.13.2...v0.14.0

### 🚀 New features

- (settings) Load configuration from a list of commonly used locations ([!444](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/444), [#134](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/134))

### 🐛 Bug fixes

- (container) Move installed executable back to `/opt/smtp-mailer` ([!403](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/403))
- Make logo an inline attachement ([!429](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/429))

### 📦 Dependencies

- (deps) Lock file maintenance ([!399](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/399))
- (deps) Update rust crate clap to v4.5.32 ([!402](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/402))
- (deps) Update rust crate env_logger to v0.11.7 ([!401](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/401))
- (deps) Update rust crate lettre to v0.11.15 ([!400](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/400))
- (deps) Update rust crate tokio to v1.44.1 ([!406](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/406))
- (deps) Update rust crate uuid to v1.16.0 ([!409](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/409))
- (deps) Update rust crate opentalk-types-common to v0.32.1 ([!407](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/407))
- (deps) Update rust crate config to v0.15.11 ([!405](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/405))
- (deps) Lock file maintenance ([!411](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/411))
- (deps) Lock file maintenance ([!412](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/412))
- (deps) Update rust crate log to v0.4.27 ([!414](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/414))
- (deps) Update rust crate clap to v4.5.34 ([!417](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/417))
- (deps) Lock file maintenance ([!419](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/419))
- (deps) Update rust crate clap to v4.5.35 ([!420](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/420))
- (deps) Update rust crate env_logger to v0.11.8 ([!421](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/421))
- (deps) Update rust crate lapin to v2.5.2 ([!422](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/422))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.86.0 ([!423](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/423))
- (deps) Lock file maintenance ([!426](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/426))
- (deps) Update rust crate vergen to v9.0.6 ([!427](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/427))
- (deps) Update rust crate vergen-gix to v1.0.9 ([!428](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/428))
- (deps) Update rust crate opentalk-types-common to v0.32.3 ([!430](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/430))
- (deps) Lock file maintenance ([!432](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/432))
- (deps) Update rust crate anyhow to v1.0.98 ([!433](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/433))
- (deps) Lock file maintenance ([!436](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/436))
- (deps) Lock file maintenance ([!440](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/440))
- (deps) Update rust crate chrono to v0.4.41 ([!442](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/442))
- (deps) Update rust crate insta to v1.43.1 ([!443](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/443))
- (deps) Lock file maintenance ([!445](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/445))
- (deps) Update rust crate tokio to v1.45.0 ([!446](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/446))
- (deps) Lock file maintenance ([!450](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/450))
- (deps) Update rust crate opentalk-types-common to 0.34 ([!452](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/452))
- (deps) Update rust crate lettre to v0.11.16 ([!451](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/451))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.87.0 ([!453](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/453))
- (deps) Lock file maintenance ([!454](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/454))
- (deps) Update rust crate uuid to v1.17.0 ([!455](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/455))
- (deps) Update rust crate opentalk-types-common to 0.35 ([!456](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/456))
- (deps) Lock file maintenance ([!458](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/458))
- (deps) Update rust crate clap to v4.5.39 ([!459](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/459))
- (deps) Update rust crate snafu to v0.8.6 ([!460](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/460))

### ⚙ Miscellaneous

- Add pre-commit config ([!408](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/408))
- Add trailing-whitespace and end-of-file-fixer to the pre-commit config ([!418](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/418))
- (settings) Move `extra/example.toml` to `example/smtp-mailer.toml` in repository ([!444](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/444))

### Ci

- Restrict mr container tag lengh to 63 characters ([!395](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/395), [#124](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/124))
- Add container security scanning ([!390](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/390))
- Configure renovate merge request reviewers ([!415](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/415))
- Hide dependency graph in cargo-deny output ([!426](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/426))
- Ignore `RUSTSEC-2025-0021` ([!426](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/426))
- Changing to latest container scanning ([!434](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/434))

## [0.13.0] - 2025-03-05

[0.13.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.12.0...v0.13.0

### 🚀 New features

- Add ubuntu noble based container image ([!364](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/364))

### 🐛 Bug fixes

- Reduce container size and attack surface ([!364](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/364))

### 🔨 Refactor

- (ci) Clean up install step in Dockerfiles ([!364](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/364))

### 📦 Dependencies

- (deps) Lock file maintenance ([!351](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/351))
- (deps) Update rust crate clap to v4.5.28 ([!353](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/353))
- (deps) Update rust crate uuid to v1.13.1 ([!354](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/354))
- (deps) Update rust crate bytes to v1.10.0 ([!352](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/352))
- (deps) Lock file maintenance ([!356](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/356))
- (deps) Update rust crate config to v0.15.8 ([!357](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/357))
- (deps) Update rust crate clap to v4.5.29 ([!358](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/358))
- (deps) Lock file maintenance ([!359](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/359))
- (deps) Update rust crate opentalk-types-common to v0.31.1 ([!365](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/365))
- (deps) Update rust crate uuid to v1.13.2 ([!363](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/363))
- (deps) Update rust crate clap to v4.5.30 ([!362](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/362))
- (deps) Update rust crate lettre to v0.11.13 ([!360](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/360))
- (deps) Update rust crate serde to v1.0.218 ([!368](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/368))
- (deps) Update rust crate serde_json to v1.0.139 ([!366](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/366))
- (deps) Update rust crate log to v0.4.26 ([!370](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/370))
- (deps) Update rust crate uuid to v1.14.0 ([!369](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/369))
- (deps) Lock file maintenance ([!372](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/372))
- (deps) Update rust crate uuid to v1.15.0 ([!375](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/375))
- (deps) Update rust crate uuid to v1.15.1 ([!378](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/378))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.85.0 ([!377](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/377))
- (deps) Update rust crate chrono to v0.4.40 ([!376](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/376))
- (deps) Update rust crate opentalk-types-common to 0.32 ([!379](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/379))
- (deps) Lock file maintenance ([!383](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/383))
- (deps) Lock file maintenance ([!388](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/388))
- (deps) Update rust crate serde_json to v1.0.140 ([!389](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/389))
- (deps) Update rust crate rstest to 0.25.0 ([!382](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/382))
- (deps) Update rust crate textwrap to v0.16.2 ([!391](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/391))
- (deps) Update rust crate config to v0.15.9 ([!392](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/392))
- (deps) Update rust crate bytes to v1.10.1 ([!393](https://git.opentalk.dev/opentalk/smtp-mailer/-/merge_requests/393))

## [0.12.0] - 2025-01-31

[0.12.0]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.11.0...v0.12.0

### 🐛 Bug fixes

- Use connection-level TLS for `smtps://` connections ([!291](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/291), [#116](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/116))
- (templates) Make ad-hoc wording more consistent ([!340](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/340))

### 🔨 Refactor

- Use opentalk-version crate ([!294](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/294))
- Add opentalk-mail-worker-protocol to smtp-mailer repository ([!314](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/314), [#118](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/118))

### 📦 Dependencies

- (deps) Lock file maintenance ([!290](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/290))
- (deps) Update rust crate thiserror to v2.0.8 ([!293](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/293))
- (deps) Lock file maintenance ([!299](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/299))
- (deps) Update rust crate rstest to 0.24.0 ([!303](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/303))
- (deps) Update rust crate async-trait to v0.1.84 ([!304](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/304))
- (deps) Update rust crate fluent-templates to 0.12.0 ([!300](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/300))
- (deps) Update rust crate clap to v4.5.24 ([!306](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/306))
- (deps) Update rust crate async-trait to v0.1.85 ([!305](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/305))
- (deps) Lock file maintenance ([!310](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/310))
- (deps) Update rust crate tokio to v1.43.0 ([!311](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/311))
- (deps) Update rust crate vergen to v9.0.3 ([!312](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/312))
- (deps) Update rust crate thiserror to v2.0.10 ([!315](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/315))
- (deps) Update rust crate vergen-gix to v1.0.4 ([!313](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/313))
- (deps) Update rust crate uuid to v1.11.1 ([!319](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/319))
- (deps) Update rust crate clap to v4.5.26 ([!318](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/318))
- (deps) Update rust crate fluent-templates to v0.12.1 ([!316](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/316))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.84.0 ([!320](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/320))
- (deps) Update rust crate log to v0.4.24 ([!323](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/323))
- (deps) Update rust crate thiserror to v2.0.11 ([!322](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/322))
- (deps) Update rust crate vergen-gix to v1.0.6 ([!326](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/326))
- (deps) Update rust crate phonenumber to v0.3.7 ([!324](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/324))
- (deps) Lock file maintenance ([!328](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/328))
- (deps) Update rust crate fluent-templates to 0.13.0 ([!327](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/327))
- (deps) Update rust crate opentalk-types-common to 0.30.0 ([!329](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/329))
- (deps) Update rust crate log to v0.4.25 ([!331](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/331))
- (deps) Update rust crate uuid to v1.12.0 ([!332](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/332))
- (deps) Update rust crate opentalk-types-common to v0.30.1 ([!333](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/333))
- (deps) Lock file maintenance ([!335](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/335))
- (deps) Update rust crate config to 0.15 ([!292](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/292))
- (deps) Update rust crate uuid to v1.12.1 ([!337](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/337))
- (deps) Update rust crate clap to v4.5.27 ([!336](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/336))
- (deps) Lock file maintenance ([!339](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/339))
- (deps) Update rust crate serde_json to v1.0.138 ([!342](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/342))
- (deps) Update rust crate config to v0.15.7 ([!344](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/344))
- (deps) Update rust crate opentalk-types-common to 0.31.0 ([!345](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/345))
- (deps) Update rust crate service-probe to v0.2.1 ([!346](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/346))

### ⚙ Miscellaneous

- Add developer documentation and corresponding CI checks ([!317](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/317))

## [0.11.0] - 2024-12-12

[0.11.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.10...v0.11.0

### 🚀 New features

- Add liveness endpoint ([!263](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/263))

### 🐛 Bug fixes

- Double quoted font-names inside double quoted style attribute ([!257](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/257))

### 📦 Dependencies

- (deps) Lock file maintenance ([!251](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/251), [!257](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/257), [!265](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/265), [!269](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/269), [!273](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/273), [!283](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/283))
- (deps) Update alpine docker tag to v3.21 ([!281](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/281))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.83.0 ([!274](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/274))
- (deps) Update opentalk-controller to 0.28.0 ([!287](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/287))
- (deps) Update rust crate anyhow to v1.0.94 ([!277](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/277))
- (deps) Update rust crate chrono to v0.4.39 ([!284](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/284))
- (deps) Update rust crate clap to v4.5.22 ([!278](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/278))
- (deps) Update rust crate css-inline to v0.14.3 ([!262](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/262))
- (deps) Update rust crate lettre to v0.11.11 ([!279](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/279))
- (deps) Update rust crate serde to v1.0.216 ([!285](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/285))
- (deps) Update rust crate thiserror to v2.0.4 ([!276](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/276))
- (deps) Update rust crate tokio to v1.42.0 ([!275](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/275))
- (deps) Update rust crate types to 0.25.0 ([!245](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/245))

### Ci

- (cargo-deny) Fail cargo-deny on warnings ([!287](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/287))

## [0.10.0]

[0.10.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.9.0...v0.10.0

### 📦 Dependencies

- Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.82.0 ([!228](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/228))
- Update opentalk-controller to 0.25.0 ([!244](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/244))
- Update rust crate anyhow to v1.0.91 ([!237](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/237))
- Update rust crate bytes to v1.8.0 ([!233](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/233))
- Update rust crate config to v0.14.1 ([!240](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/240))
- Update rust crate lettre to v0.11.10 ([!239](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/239))
- Update rust crate serde to v1.0.214 ([!234](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/234), [!236](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/236), [!243](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/243))
- Update rust crate serde_json to v1.0.129 ([!227](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/227))
- Update rust crate thiserror to v1.0.65 ([!238](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/238))
- Update rust crate tokio to v1.41.0 ([!235](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/235))
- Update rust crate uuid to v1.11.0 ([!226](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/226))
- Lock file maintenance ([!231](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/231), [!225](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/225), [!242](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/242))

### ⚙ Miscellaneous

- Sync changelog with version 0.9.0 ([!223](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/223))

### Ci

- Introduce changelog bot ([!232](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/232))

## [0.9.0]

[0.9.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.8.0...v0.9.0

### 🚀 New features

- Expose preview functionality in library ([!215](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/215))
- Stable preview output ([!215](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/215))

### 🐛 Bug fixes

- (ci) Build of container when `vendored` dir is gone ([!163](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/163))

### 📦 Dependencies

- (deps) Ignore RUSTSEC-2024-0370 ([!192](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/192))
- (deps) Lock file maintenance ([!170](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/170), [!175](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/175), [!183](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/183), [!184](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/184), [!188](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/188), [!189](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/189), [!202](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/202), [!210](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/210), [!213](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/213), [!217](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/217))
- (deps) Update chrono-tz ([!190](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/190))
- (deps) Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.81.0 ([!191](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/191))
- (deps) Update opentalk-controller to 0.20.0 ([!206](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/206))
- (deps) Update rust crate anyhow to v1.0.88 ([!193](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/193))
- (deps) Update rust crate async-trait to v0.1.83 ([!212](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/212))
- (deps) Update rust crate bytes to v1.7.2 ([!205](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/205))
- (deps) Update rust crate clap to v4.5.20 ([!219](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/219))
- (deps) Update rust crate env_logger to v0.11.5 ([!153](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/153))
- (deps) Update rust crate fluent-templates to 0.11.0 ([!208](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/208))
- (deps) Update rust crate gix-path to 0.10.11 ([!192](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/192))
- (deps) Update rust crate lapin to v2.5.0 ([!156](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/156))
- (deps) Update rust crate lettre to v0.11.8 ([!196](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/196))
- (deps) Update rust crate pest ([!139](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/139))
- (deps) Update rust crate pretty_assertions to v1.4.1 ([!201](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/201))
- (deps) Update rust crate serde to v1.0.210 ([!197](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/197))
- (deps) Update rust crate serde_json to v1.0.128 ([!198](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/198))
- (deps) Update rust crate snafu to v0.8.5 ([!211](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/211))
- (deps) Update rust crate tokio to v1.39.2 ([!155](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/155))
- (deps) Update rust crate types-common to 0.21.0 ([!220](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/220))
- (deps) Update rust crate vergen to v9 ([!139](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/139))
- (deps) Update rust crate vergen-gix to v1.0.2 ([!204](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/204))
- (deps) Update rust docker tag to v1.80.0 ([!154](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/154))

### ⚙ Miscellaneous

- Replace controller submodule by crates.io dependencies ([!161](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/161))
- (renovate) Group opentalk-controller libraries ([!171](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/171))
- (ci) Build with cargo-auditable ([!167](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/167))
- (ci) Add container mr build ([!167](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/167))

### Test

- Snapshot tests for preview output ([!215](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/215))

## [0.8.0]

[0.8.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.7.0...v0.8.0

### 🐛 Bug Fixes

- (deps) Update rust crate async-trait to v0.1.81
- (deps) Update rust crate serde to v1.0.204
- (deps) Update rust crate clap to v4.5.9
- (deps) Update rust crate uuid to v1.10.0
- (deps) Update rust crate phonenumber to v0.3.6
- (deps) Update rust crate thiserror to v1.0.62
- (deps) Update rust crate bytes to v1.6.1
- (deps) Update rust crate tokio to v1.38.1
- (deps) Update rust crate lapin to v2.4.0
- (deps) Update rust crate thiserror to v1.0.63

### 📚 Documentation

- Add a general description and link the protocol docs

## [0.7.0]

[0.7.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.6.0...v0.7.0

### 🚀 New features

- Add event details to email templates ([#85](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/85))

### ⚙ Dependencies

- Update controller submodule to version 0.16.0-rc.1
- Update git.opentalk.dev:5050/opentalk/backend/containers/rust docker tag to v1.79.0
- Update rust crate anyhow to v1.0.86
- Update rust crate chrono-tz to 0.9
- Update rust crate clap to v4.5.8
- Update rust crate css-inline to 0.14.0
- Update rust crate fluent-templates to 0.9.0
- Update rust crate ics-chrono-tz to 0.3
- Update rust crate lapin to v2.3.4
- Update rust crate lettre to 0.11.0
- Update rust crate log to v0.4.22
- Update rust crate phonenumber to v0.3.5
- Update rust crate serde to v1.0.203
- Update rust crate serde_json to v1.0.119
- Update rust crate tera to v1.20.0
- Update rust crate thiserror to v1.0.61
- Update rust crate tokio to v1.38.0
- Update rust crate url to v2.5.2
- Update rust crate uuid to v1.9.1

### Ci

- Lint commits, md, yaml and format tomls

## [0.6.0]

[0.6.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.5.2...v0.6.0

### 🚀 New features

- Added previews for: registered-event-update and registered-uninvite
- Add streaming information to all templates ([#66](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/66))

### 🐛 Bug fixes

- Don't show passwords for registered users ([#93](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/93))
- print Version output ([!92](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/merge_requests/91))

### Ci

- Upgrade debian image in ci & container creation to bookworm ([#92](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/92))

### :gear: Miscellaneous Tasks

- Update controller submodule to v0.15.0

## [0.5.2]

[0.5.2]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.5.1...v0.5.2

### Fixed

- `RUSTSEC-2024-0336` by updating dependencies ([#94](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/94))

## [0.5.1]

[0.5.1]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/v0.5.0...v0.5.1

### Changed

- Update dependencies ([#88](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/88))
- Use ics-chrono-tz from crates.io ([#87](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/87))

## [0.5.0] - 2024-02-22

[0.5.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/fb7bb1046c6850b5bf8f42f7ab58a4eab68d7ad3...v0.5.0

### Changed

- Create a proper ICS file for event instance updates ([#60](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/60))

## [0.4.3] - 2024-01-10

[0.4.3]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.4.2...v0.4.3

### Added

- Add deletion notice for adhoc meetings to emails ([#57](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/57))

### Fixed

- Update `self_cell` dependency to fix [`RUSTSEC-2023-0070`](https://rustsec.org/advisories/RUSTSEC-2023-0070) ([#76](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/76))

## [0.4.2] - 2023-11-02

[0.4.2]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.4.1...v0.4.2

### Fixed

- Pin dockerfile build container to Debian Bullseye to fix GLIBC errors ([#72](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/72))

## [0.4.1] - 2023-11-02

[0.4.1]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/v0.4.0...v0.4.1

### Fixed

- Pin build base container to Debian Bullseye to fix GLIBC errors ([#70](https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/issues/70))

## [0.4.0] - 2023-10-30

[0.4.0]: https://git.opentalk.dev/opentalk/backend/services/smtp-mailer/-/compare/f51209ff8fc6d709c3df81198bf709b88a64f44d...v0.4.0

### Added

- Add quick guide hint (including link) to the generated emails ([#65](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/65))
- Send emails to users when they are removed from a meeting ([#48](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/48))

### Changed

- Change wording in emails to match the voice announcements ([#46](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/46))
- Add quick guide hint (including link) to the generated emails ([#65](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/65))
- Update CI rust tooling to 1.73.0  ([#62](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/62))

## [0.3.0] - 2023-06-27

[0.3.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/25b0d812a39f1cfb6b5dd11598a7be9e20964eda...v0.3.0

### Added

- Include shared folder URL and password in emails ([#34](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/34))
- Data protection hints ([#39](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/39))

### Fixed

- Make shared folder link clickable ([#41](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/41))
- Replace deprecated lobby URLs with room counterparts ([#61](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/61))

## [0.2.0] - 2023-05-15

[0.2.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/ffadbb4aacd48a91eb0553dba966745476640941...v0.2.0

### Changed

- The creator is now referenced via Reply-To instead of From header field, From is configurable ([#33](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/33))

### Fixed

- Show the inviter's full name, not just the first name ([#31](https://git.opentalk.dev/opentalk/smtp-mailer/-/issues/31))

## [0.1.0] - 2023-03-01

[0.1.0]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/14c20df5d3a54c39332287ebf719ec04b49d4bab...v0.1.0

### Added

- Add license information

### Fixed

- fixed a bug where environment variables did not overwrite config values

## [0.0.0-internal-release.4] - 2022-11-10

[0.0.0-internal-release.4]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/7579c621c3b08c086974e7ccf4365345762e69c8...14c20df5d3a54c39332287ebf719ec04b49d4bab

### Added

- Added support for `...EventUpdate` and `...EventCancellation` messages
- First name and last name are now available for unregistered users as well

### Changed

- changed internationalization strings for unregistered and external invitees
- exit process on IO errors or when the SMTP-server is no longer available

### Fixed

- do not requeue emails when encountering permanent errors

## [0.0.0-internal-release.3] - 2022-07-20

[0.0.0-internal-release.3]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/8f7edbcb83ac2b38402ebc951f76b75ddc51043d...7579c621c3b08c086974e7ccf4365345762e69c8

### Changed

- internal dependency updates

## [0.0.0-internal-release.2] - 2022-07-20

[0.0.0-internal-release.2]: https://git.opentalk.dev/opentalk/smtp-mailer/-/compare/5de6fd35a071b05cd05fd03838d436bd9c79db53...8f7edbcb83ac2b38402ebc951f76b75ddc51043d

### Fixed

- Improved outlook support by fixing the head section and adding special http-equiv meta tags (opentalk/smtp-mailer!18)
- Added timezone information to ics files
- Fixed missing newline in call-in sections
- Added timezone to dateformatter to return the correct time
- disabled bidi isolation unicode characters in fluent output

## [0.0.0-internal-release.1] - 2022-06-23

[0.0.0-internal-release.1]: https://git.opentalk.dev/opentalk/smtp-mailer/-/commits/5de6fd35a071b05cd05fd03838d436bd9c79db53

initial release candidate
