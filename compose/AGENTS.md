<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Agent instructions for opentalk-compose

Rules for AI coding agents working in this repository. Follow them literally.

Keep this file under 120 lines. Never restate a number that lives in another file — link to that file instead.

## What this repository is

- Deployment automation for OpenTalk. Ansible roles render Docker Compose files; there is no application code here.
- Shipped as `.deb` and `.rpm` containing an embedded Python plus Ansible runtime and the `otctl` CLI.
- Behaviour changes belong in Ansible tasks, Jinja templates, or the shell scripts under `extras/`.
- CI is GitLab CI. Documentation is MkDocs.

## Commands

Run `just --list` for the full set. Use these commands; do not invent your own.

| Task | Command |
| --- | --- |
| Lint everything (read-only) | `just lint` |
| Lint plus all fast tests | `just test-fast` |
| Run one fast test | `bash extras/scripts/tests/<name>.sh` |
| Enforce coverage thresholds | `just test-coverage` |
| Measure coverage of one test | `just test-coverage-one extras/scripts/tests/<name>.sh` |
| Regenerate the config reference | `python3 extras/scripts/generate-config-reference.py` |
| Build the documentation | `just docs` |
| Deploy locally (needs Docker) | `just compose-setup && just compose-deploy` |
| Stack status and logs | `just compose-status`, `just compose-logs [service]` |
| Stop the local stack | `just compose-down` |
| Build packages | `just build-deb`, `just build-rpm` |

Destructive. Run only when the user explicitly asks: `just compose-clean`, `just clean`, `just clean-artifacts`, `just clean-all`, `just test-e2e`.

## Hard rules

- Every new file needs an SPDX header naming `OpenTalk GmbH <mail@opentalk.eu>` and `EUPL-1.2`.
  - `.sh`, `.py`, `.yml`, `.j2`: a `#` comment block. In YAML it goes before the `---`.
  - `.md`: an HTML comment block with a blank line between the two tags. Never use `#` — that adds a second H1 and fails `mdl`.
  - Formats without comments, such as JSON: add the path to `REUSE.toml` instead.
- Never hand-edit `docs/6-configuration-options-reference.md`. It is generated. Edit the role's `defaults/main.yml`, then regenerate and commit the result.
- Never commit: `build/`, `artifacts/`, `site/`, `.ansible/`, `config/`, `*.deb`, `*.rpm`, `extras/packaging/requirements-ansible.txt`.
- Commit messages follow Conventional Commits: lowercase subject, no trailing period, at most 100 characters. CI rejects violations.
- Every bugfix that can be covered by a test gets a regression test in the same change.
- Start tests with `bash <file>`, not `./<file>`. Most test files have no exec bit.
- Never run the test suite as root. `test_opentalk_build_deps.sh` skips a branch as root and its coverage gate then fails.
- `.versions.yml` is the single source of truth for component versions. Put every new pin there and never add a second place for one. Read it with `versions_get` from `extras/packaging/lib/common.sh` in shell, or with `yq` in CI. Never re-implement the parsing.
- A new pin needs a Renovate annotation above it. If it genuinely cannot be automated, leave the annotation out and write a comment saying why.
- Package manifests stay where their tooling expects them: `extras/packaging/requirements-ansible.in` for pip-compile, `automation/collections/requirements.yml` for ansible-galaxy.

## Layout

- `automation/roles/opentalk_compose_<service>/` — one role per service. `defaults/main.yml` holds the public variables, `tasks/main.yml` only dispatches via `include_tasks`, `templates/compose.yml.j2` renders the service.
- `automation/playbooks/opentalk_compose.yml` — the main playbook. It reads `OT_SETTINGS_FILE` and `.versions.yml`.
- `extras/packaging/` — numbered build stages `10-` through `90-`, orchestrated by `build-all.sh`. `wrappers/` holds `otctl` and the `ot-ansible*` wrappers, `lib/common.sh` the shared shell functions.
- `extras/scripts/tests/` — fast tests, no Docker and no network. `extras/scripts/tests/integration/` needs Docker.
- `extras/scripts/coverage/` — the coverage measurement tools.
- `docs/` — MkDocs sources. Adding a chapter also means editing `docs/navigation.md` by hand.
- `services/` — empty placeholder, filled at deploy time.

## Ansible conventions

- Always use fully qualified module names, for example `ansible.builtin.template` instead of `template`. `ansible-lint` runs the `production` profile.
- Task names start with a capital letter.
- Public role variables use a short service prefix such as `controller_` or `mailer_`, not the full role name.
- Internal registered facts start with an underscore, for example `_blockinfile_result`.
- Secrets are `SCREAMING_SNAKE_CASE` and declared with their length in `automation/roles/opentalk_compose_secrets/tasks/manage_secrets.yml`. Never use a secret directly in a template; map it to a lowercase role variable in `defaults/main.yml` first.
- Every `command` task needs `changed_when`. Every file or template task needs an explicit `mode`.
- Suppress a lint rule only inline with `# noqa`, never by widening `.ansible-lint.yml`.

## Writing tests

- New fast tests go into `extras/scripts/tests/` as `test_*.sh` and are picked up automatically.
- Follow the existing skeleton: `set -euo pipefail`, `REPO_ROOT` derived from `BASH_SOURCE`, `mktemp -d` with a `trap` cleanup, and a final `echo "... tests passed."`.
- Point the script under test at the temporary directory using its environment override, such as `PACKAGING_ROOT`, `CLEANUP_ROOT`, `OT_PREFIX` or `OT_BACKUP_DIR`. Without the override the script operates on the real repository.
- No network, no Docker, no host-specific paths, no root.
- Do not add a new tool dependency without asking first.
- `extras/scripts/tests/README.md` has the full inventory, the skeleton, and the environment override reference.

## Traps

- `extras/packaging/90-cleanup.sh` deletes the real `build/` and `artifacts/` unless `CLEANUP_ROOT` is set.
- `test_opentalk_ci_e2e_contract.sh` greps `.gitlab-ci.yml` and `test_compose_e2e.sh` literally. Reformatting those lines breaks the test even when the behaviour is unchanged.
- `test_opentalk_config_reference_generator.sh` requires the generated reference to match the committed one byte for byte.
- Fast tests that call `ansible-playbook` need the vendored collections. Run `just compose-collections` once; it needs network access.
- The fast test runner uses `xargs`, which exits with 123 and keeps going after a failure. Read the output, do not trust the exit code alone.
- `just test-fast` runs `just lint` first, so it needs `pre-commit`, `shellcheck`, `yamllint`, `mdl`, `reuse` and `ansible-lint` on the host.

## Coverage

- `extras/scripts/tests/test_opentalk_coverage_regression.sh` is the authoritative list of coverage thresholds. Read the current values there.
- Baselines only ever go up. Never lower a threshold to make a change pass.
