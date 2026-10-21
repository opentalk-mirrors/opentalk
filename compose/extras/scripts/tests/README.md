<!--
SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>

SPDX-License-Identifier: EUPL-1.2
-->

# Test suite

Reference for the test suite of `opentalk-compose`. The binding rules for agents and contributors live in `AGENTS.md` at the repository root; this file holds the detail behind them.

## Running tests

```bash
just test-fast                                              # lint + every fast test
bash extras/scripts/tests/test_opentalk_cleanup.sh          # a single fast test
just test-coverage                                          # enforce the coverage thresholds
just test-coverage-one extras/scripts/tests/test_x.sh       # coverage of one test
```

To skip the `just lint` dependency of `test-fast` and only run the tests, invoke the runner directly:

```bash
find extras/scripts/tests -maxdepth 1 -type f -name 'test_*.sh' -print0 | xargs -0r -n1 bash
```

CI runs the same command in the `lint:shell-tests` job.

## Fast tests

No Docker, no network. Discovered automatically by filename pattern `test_*.sh` at depth 1.

| Test | Covers |
| --- | --- |
| `test_opentalk_build_all.sh` | `build-all.sh` orchestration against stubbed stages |
| `test_opentalk_build_deps.sh` | `install-build-deps.sh` check and install dispatch |
| `test_opentalk_check_packaging.sh` | `check-packaging.sh` under a strict fake `PATH` |
| `test_opentalk_ci_e2e_contract.sh` | grep contract for the `int:e2e` job and E2E isolation |
| `test_opentalk_cleanup.sh` | `90-cleanup.sh`, all modes, via `CLEANUP_ROOT` |
| `test_opentalk_compose_templates_render.sh` | renders every `compose.yml.j2`, validates with `yaml.safe_load` |
| `test_opentalk_compose_templates_yaml.sh` | `---` and `services:` consistency of the templates |
| `test_opentalk_config_reference_generator.sh` | generated reference equals the committed file |
| `test_opentalk_coverage_regression.sh` | enforces the shell and Python coverage thresholds |
| `test_opentalk_dev_settings_routing.sh` | dev settings: containers reach the dev URLs via Traefik aliases, not `host-gateway` |
| `test_opentalk_import_users.sh` | CSV and JSON import, password policy |
| `test_opentalk_keycloak_idempotency.sh` | idempotency guards of the realm task chain |
| `test_opentalk_keycloak_permanent_admin.sh` | permanent admin contract |
| `test_opentalk_keycloak_realm_export_sanitize.sh` | realm export sanitizer, `jq` assertions |
| `test_opentalk_migrations.sh` | migration detection, deploy guard and `otctl migrate status` |
| `test_opentalk_orchestrator_storage_config.sh` | regression: rendered `orchestrator.toml` always has a `[storage]` section |
| `test_opentalk_otctl_smoke.sh` | `otctl` CLI flows against a fake `OT_PREFIX` |
| `test_opentalk_packaging_common_lib.sh` | functions in `extras/packaging/lib/common.sh` |
| `test_opentalk_packaging_maintainer_scripts.sh` | `pkg-postinst.sh`, `pkg-postrm.sh`, `pkg-prerm.sh` |
| `test_opentalk_packaging_stages_build.sh` | stages `20-` through `50-` |
| `test_opentalk_packaging_stages_package.sh` | stages `60-` through `80-` |
| `test_opentalk_packaging_stages_prepare.sh` | stages `10-` through `14-` |
| `test_opentalk_playbook_check_mode.sh` | `ansible-playbook --check` with synthetic settings |
| `test_opentalk_role_contracts.sh` | role references resolve, no legacy dotted role dirs |
| `test_opentalk_s3_migrate.sh` | S3 migration flow and settings update |
| `test_opentalk_secrets_and_certificates.sh` | secrets role and self-signed certificates |
| `test_opentalk_smtp_mailer_process_limit.sh` | regression: no `nproc` in the mailer template |
| `test_opentalk_verify_packages.sh` | `verify-packages.sh` arguments and metadata checks |
| `test_opentalk_wrappers_ansible.sh` | the `ot-ansible*` wrappers |

Shared Ansible fixtures live in `fixtures/`.

Some fast tests need host tools and abort without them: `ansible-playbook` plus populated `.ansible/collections` (run `just compose-collections` once), `jq`, and `python3` with PyYAML. `test_opentalk_verify_packages.sh` skips cleanly with exit 0 when `dpkg-deb` or `rpm` is missing.

## Integration tests

Need Docker.

| Test | Notes |
| --- | --- |
| `integration/test_deployment_runtime.sh` | container health per Compose project, runs in CI across the distro matrix |
| `integration/test_smtp_mailer_rabbitmq.sh` | mailer and RabbitMQ, CI runs it in experimental pipelines only |
| `integration/test_compose_e2e.sh` | destructive full-stack run, requires `OPENTALK_E2E=1`, entry point `just test-e2e` |
| `integration/test_backup_restore_config.sh` | backup and restore lifecycle, requires `OPENTALK_E2E=1` and an existing dev stack |
| `integration/test_postgres_migration.sh` | `otctl migrate postgres` against real containers, including rollback after a failure; requires `OPENTALK_E2E=1` and root (password files as the package writes them), isolates itself with its own project name; with `OT_PREFIX` it tests the installed package, CI job `int:postgres-migration` |
| `integration/test_upgrade_e2e.sh` | upgrade from the current stable package to the package under test, deploy guard, `otctl migrate postgres`, data survives; destructive, root on a disposable Debian host, requires `OPENTALK_E2E=1`, CI job `int:upgrade` |

The destructive tests exit 2 without `OPENTALK_E2E=1`. The E2E test isolates itself from the local dev stack with its own project name, directory and ports.

## Fast test skeleton

```bash
#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# What is under test, and why this run is host independent: which script,
# which env override points it at the temp dir, which tools are stubbed.

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
TARGET="$REPO_ROOT/extras/packaging/<script>.sh"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
ROOT="$TMP_DIR/root"
FAKE_TOOLS="$TMP_DIR/tools"
mkdir -p "$ROOT" "$FAKE_TOOLS"

# Host independence: point the script at the temp tree, never the real repo.
export PACKAGING_ROOT="$ROOT"
export SOURCE_DATE_EPOCH=1700000000

# Stubs plus real symlinks in a strict PATH, so nothing falls back to the host.
make_stub() { printf '#!/usr/bin/env bash\nexit 0\n' > "$FAKE_TOOLS/$1"; chmod +x "$FAKE_TOOLS/$1"; }
for tool in bash env dirname grep sed date; do ln -s "$(command -v "$tool")" "$FAKE_TOOLS/$tool"; done

run_stage() {
  local expect_rc=$1
  set +e
  (cd "$ROOT" && PATH="$FAKE_TOOLS" bash "$TARGET") >"$TMP_DIR/log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "expected rc $expect_rc, got $rc" >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

log_has() { grep -q "$1" "$TMP_DIR/log" || { echo "log missing: $1" >&2; cat "$TMP_DIR/log" >&2; exit 1; }; }

# ... test cases ...

echo "<topic> tests passed."
```

Integration tests use `../../../..` for `REPO_ROOT` because of the extra directory level.

## Environment overrides

Every script that touches the filesystem takes an override so a test can redirect it into a temp directory. Using the override is mandatory — without it the script operates on the real repository or on `/opt`.

| Variable | Read by | Effect |
| --- | --- | --- |
| `PACKAGING_ROOT` | `lib/common.sh` and every numbered stage | relocates the whole build tree, including `build/` and `artifacts/` |
| `VERSIONS_FILE` | `versions_get` and `versions_lookup` in `lib/common.sh` | which `.versions.yml` to read; derives from `PACKAGING_ROOT`, so a fake root needs its own copy |
| `CLEANUP_ROOT` | `90-cleanup.sh` | relocates the deletion target; without it the real `build/` and `artifacts/` are deleted |
| `OT_PREFIX` | `otctl`, the `ot-ansible*` wrappers, the maintainer scripts | install root, defaults to `/opt/opentalk-compose` |
| `OT_BACKUP_DIR` | `otctl` backup and restore | backup root |
| `OT_SYSTEM_BACKUP_DIR` | `otctl` backup and restore | fallback backup root when `OT_BACKUP_DIR` is unset, defaults to `/var/opt/opentalk-compose/backups` |
| `OT_CFG_DIR` | `pkg-postinst.sh` | config directory, defaults to `/etc/opentalk-compose` |
| `OT_EXAMPLES_DIR` | `pkg-postinst.sh` | example files directory |
| `OT_SETTINGS_FILE` | the main playbook | settings source |
| `OPENTALK_DEV_DIR` | `justfile`, backup and restore test | local dev deployment directory |
| `SOURCE_DATE_EPOCH` | `lib/common.sh` and the packaging stages | reproducible timestamps |
| `FORCE_REFRESH` | the fetch, lock and vendor stages | bypasses all hash and version caches |
| `OPENTALK_E2E` | the destructive integration tests | mandatory guard, exit 2 without it |
| `OT_UPGRADE_PACKAGE` | `test_upgrade_e2e.sh` | the `.deb` under test; defaults to the newest in `artifacts/` |
| `OT_UPGRADE_FROM_VERSION` | `test_upgrade_e2e.sh` | package version to upgrade from; defaults to the newest stable release |
| `OT_UPGRADE_EXTRA_SETTINGS` | `test_upgrade_e2e.sh` | YAML file appended to the CI settings, for host-specific overrides |
| `OPENTALK_E2E_PROJECT_NAME` | `test_compose_e2e.sh` | isolated Compose project name |
| `OPENTALK_E2E_DEV_DIR` | `test_compose_e2e.sh` | isolated deployment directory |
| `OPENTALK_E2E_INGRESS_PORT` | `test_compose_e2e.sh` | isolated ingress port |
| `OPENTALK_E2E_KEYCLOAK_PORT` | `test_compose_e2e.sh` | isolated Keycloak port |
| `COMPOSE_PROJECT_NAME` | the runtime integration tests | which Compose project to inspect |
| `OPENTALK_HEALTH_TIMEOUT_SECONDS` | `test_deployment_runtime.sh` | health wait timeout |

The defaults of the `OPENTALK_E2E_*` variables are pinned by `test_opentalk_ci_e2e_contract.sh`. Changing one of them means updating that contract in the same change.

## Coverage

`extras/scripts/coverage/measure_shell_coverage.py` traces a test with `bash -x` and reports line coverage of the script under test, including sourced libraries. Pure control-flow lines, function definition lines, multi-line continuations and here-doc bodies are excluded from the denominator, because bash never traces them.

`extras/scripts/tests/test_opentalk_coverage_regression.sh` is the authoritative list of thresholds. The values there are baselines from the moment each gate was introduced; the goal is growth over time. Never lower a threshold to make a change pass.

Do not run the suite as root. `test_opentalk_build_deps.sh` skips its "refuses to install without sudo" branch when running as root, which lowers measured coverage below the gate.

## Deliberately untested

These paths are excluded on purpose. Do not spend effort covering them without discussing it first.

- Colour output in `otctl` and the wrappers, which depends on an attached TTY.
- The interactive restore confirmation prompt in `otctl`.
- The `python3` fallback in `otctl`, which is only reachable when `jq` is absent.
- Residual gaps in the packaging stages that stem from measurement artefacts: `$(...)` command substitutions, `find -quit`, multi-line array assignments and here-doc closers are attributed to a single line by `bash -x`.
