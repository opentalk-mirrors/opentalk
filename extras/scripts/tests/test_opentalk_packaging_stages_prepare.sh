#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Stage scripts 10-14 (prepare chain): ansible-lock, uv-check,
# python-runtime-fetch, tools-fetch, runtime-compatibility-check. Each script
# reads build inputs from $PACKAGING_ROOT, which the test points at a temp
# "repo" it builds itself. External tools (uv, curl, unzip, file, objdump,
# readelf, python) are provided as deterministic stubs in a strict PATH.

set -euo pipefail

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
STAGES="$REPO_ROOT/extras/packaging"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
ROOT="$TMP_DIR/root"
FAKE_TOOLS="$TMP_DIR/tools"
export PACKAGING_ROOT="$ROOT"
export SOURCE_DATE_EPOCH=1700000000

mkdir -p "$ROOT/build" "$ROOT/extras/packaging"
mkdir -p "$FAKE_TOOLS"

log_has() {
  grep -q "$1" "$TMP_DIR/log" || { echo "log missing: $1" >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

# --- fake tool helpers ------------------------------------------------------

make_stub() {
  printf '#!/usr/bin/env bash\nexit 0\n' > "$FAKE_TOOLS/$1"
  chmod +x "$FAKE_TOOLS/$1"
}

make_uv_stub() {
  # uv <cmd> --version emulation; $1 controls the reported version.
  cat > "$FAKE_TOOLS/uv" <<EOF
#!/usr/bin/env bash
if [[ "\$1" == "--version" ]]; then
  echo "uv ${UV_VERSION_STUB:-0.5.0}"
fi
exit 0
EOF
  chmod +x "$FAKE_TOOLS/uv"
}

build_fake_path() {
  # Symlink the real tools the stage script may call; overrides go on top.
  rm -rf "$FAKE_TOOLS"
  mkdir -p "$FAKE_TOOLS"
  for tool in bash env dirname pwd echo cat rm mkdir cp mv chmod ln find \
      sort tail printf awk grep sed tr sha256sum install mktemp date; do
    if command -v "$tool" >/dev/null 2>&1; then
      ln -s "$(command -v "$tool")" "$FAKE_TOOLS/$tool"
    fi
  done
}

run_stage() {
  local script=$1
  local expect_rc=$2
  shift 2
  set +e
  PATH="$FAKE_TOOLS" bash "$STAGES/$script" "$@" >"$TMP_DIR/log" 2>&1
  local rc=$?
  set -e
  [[ $rc -eq $expect_rc ]] || { echo "$script expected rc $expect_rc, got $rc." >&2; cat "$TMP_DIR/log" >&2; exit 1; }
}

# ===========================================================================
# 10-ansible-lock.sh
# ===========================================================================

# Missing input file aborts.
build_fake_path
rm -rf "$ROOT"
mkdir -p "$ROOT/build" "$ROOT/extras/packaging"
run_stage 10-ansible-lock.sh 1
log_has 'Missing'

# Up-to-date path: matching hash in state file exits 0 without touching python.
build_fake_path
mkdir -p "$ROOT/build/.cache"
printf 'lock-input\n' > "$ROOT/extras/packaging/requirements-ansible.in"
printf 'lock-output\n' > "$ROOT/extras/packaging/requirements-ansible.txt"
printf '%s' "$(sha256sum "$ROOT/extras/packaging/requirements-ansible.in" | awk '{print $1}')" \
  > "$ROOT/build/.cache/requirements-ansible.in.sha256"
run_stage 10-ansible-lock.sh 0
log_has 'Up-to-date (no changes in'
[[ -f "$ROOT/extras/packaging/requirements-ansible.txt" ]] || { echo "OUT must remain." >&2; exit 1; }

# Regeneration: fake portable python creates a venv whose activate exposes
# pip/python stubs; the lock file is regenerated and the state hash written.
build_fake_path
mkdir -p "$ROOT/build/python/bin"
cat > "$ROOT/build/python/bin/python3" <<'PYEOF'
#!/usr/bin/env bash
if [[ "$1" == "-m" && "$2" == "venv" ]]; then
  d="${3:?}"
  mkdir -p "$d/bin"
  {
    echo "PATH=\"$d/bin:\$PATH\""
    echo "export PATH"
    echo "deactivate() { :; }"
  } > "$d/bin/activate"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$d/bin/pip"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$d/bin/python"
  chmod +x "$d/bin/pip" "$d/bin/python"
  exit 0
fi
exit 0
PYEOF
chmod +x "$ROOT/build/python/bin/python3"
printf 'lock-input-changed\n' > "$ROOT/extras/packaging/requirements-ansible.in"
run_stage 10-ansible-lock.sh 0
log_has '(Re)generating lock file'
log_has 'Wrote'
[[ -f "$ROOT/build/.cache/requirements-ansible.in.sha256" ]] || { echo "state hash must be written." >&2; exit 1; }

# Forced refresh regenerates even when state matches.
build_fake_path
mkdir -p "$ROOT/build/.cache"
printf 'lock-input\n' > "$ROOT/extras/packaging/requirements-ansible.in"
CUR=$(sha256sum "$ROOT/extras/packaging/requirements-ansible.in" | awk '{print $1}')
printf '%s' "$CUR" > "$ROOT/build/.cache/requirements-ansible.in.sha256"
mkdir -p "$ROOT/build/python/bin"
cat > "$ROOT/build/python/bin/python3" <<'PYEOF'
#!/usr/bin/env bash
if [[ "$1" == "-m" && "$2" == "venv" ]]; then
  d="${3:?}"
  mkdir -p "$d/bin"
  {
    echo "PATH=\"$d/bin:\$PATH\""
    echo "export PATH"
    echo "deactivate() { :; }"
  } > "$d/bin/activate"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$d/bin/pip"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$d/bin/python"
  chmod +x "$d/bin/pip" "$d/bin/python"
  exit 0
fi
exit 0
PYEOF
chmod +x "$ROOT/build/python/bin/python3"
FORCE_REFRESH=1 run_stage 10-ansible-lock.sh 0
log_has '(Re)generating lock file'

# ===========================================================================
# 11-uv-check.sh
# ===========================================================================

printf 'UV_VERSION=0.5.0\n' > "$ROOT/extras/packaging/uv.version"

# Missing uv aborts.
build_fake_path
run_stage 11-uv-check.sh 1
log_has 'uv 0.5.0 is required'

# Version mismatch aborts.
build_fake_path
UV_VERSION_STUB=0.4.0 make_uv_stub
run_stage 11-uv-check.sh 1
log_has 'expected uv 0.5.0, found 0.4.0'

# Matching version succeeds.
build_fake_path
UV_VERSION_STUB=0.5.0 make_uv_stub
run_stage 11-uv-check.sh 0
log_has 'Using uv 0.5.0'

# UV_BIN override is honoured.
build_fake_path
UV_VERSION_STUB=0.5.0 make_uv_stub
UV_BIN="$FAKE_TOOLS/uv" run_stage 11-uv-check.sh 0
log_has 'Using uv 0.5.0'

# ===========================================================================
# 12-python-runtime-fetch.sh
# ===========================================================================

# --version is required.
build_fake_path
run_stage 12-python-runtime-fetch.sh 1
log_has 'Python version is required'

# Unknown option aborts with rc 2.
build_fake_path
run_stage 12-python-runtime-fetch.sh 2 --bogus
log_has 'Unknown option'

# uv missing aborts.
build_fake_path
run_stage 12-python-runtime-fetch.sh 1 --version 3.12
log_has 'uv is required to download Python'

# Cached path: marker matches requested version, skip.
build_fake_path
make_uv_stub
mkdir -p "$ROOT/build/python/bin"
printf 'uv-python:3.12.4' > "$ROOT/build/python/.source-url"
run_stage 12-python-runtime-fetch.sh 0 --version 3.12.4
log_has 'Cached runtime matches source and checksum. Skip.'

# Download path: uv stub installs a cpython tree which is copied in place.
build_fake_path
rm -rf "$ROOT/build/python"
cat > "$FAKE_TOOLS/uv" <<'UVEOF'
#!/usr/bin/env bash
# uv python install <version> --install-dir <dir> --no-bin
if [[ "$1" == "python" && "$2" == "install" ]]; then
  dir=""
  prev=""
  for a in "$@"; do
    [[ "$prev" == "--install-dir" ]] && dir="$a"
    prev="$a"
  done
  mkdir -p "$dir/cpython-3.12.4/bin"
  printf '#!/usr/bin/env bash\necho "Python 3.12.4"\n' > "$dir/cpython-3.12.4/bin/python3"
  chmod +x "$dir/cpython-3.12.4/bin/python3"
  exit 0
fi
exit 0
UVEOF
chmod +x "$FAKE_TOOLS/uv"
run_stage 12-python-runtime-fetch.sh 0 --version 3.12.4
log_has 'Installing Python 3.12.4 with uv'
log_has 'Ready at'
[[ -x "$ROOT/build/python/bin/python3" ]] || { echo "python must be installed." >&2; exit 1; }
[[ "$(cat "$ROOT/build/python/.source-url")" == "uv-python:3.12.4" ]] || { echo "marker mismatch." >&2; exit 1; }

# uv creates no cpython dir -> abort.
build_fake_path
rm -rf "$ROOT/build/python"
cat > "$FAKE_TOOLS/uv" <<'UVEOF'
#!/usr/bin/env bash
# create the install dir so find succeeds, but leave it empty (no cpython-*)
if [[ "$1" == "python" && "$2" == "install" ]]; then
  dir=""
  prev=""
  for a in "$@"; do
    [[ "$prev" == "--install-dir" ]] && dir="$a"
    prev="$a"
  done
  mkdir -p "$dir"
  exit 0
fi
exit 0
UVEOF
chmod +x "$FAKE_TOOLS/uv"
run_stage 12-python-runtime-fetch.sh 1 --version 3.12.4
log_has 'uv did not create a Python installation'

# uv creates a dir but no python3 executable -> abort.
build_fake_path
rm -rf "$ROOT/build/python"
cat > "$FAKE_TOOLS/uv" <<'UVEOF'
#!/usr/bin/env bash
if [[ "$1" == "python" && "$2" == "install" ]]; then
  dir=""
  prev=""
  for a in "$@"; do
    [[ "$prev" == "--install-dir" ]] && dir="$a"
    prev="$a"
  done
  mkdir -p "$dir/cpython-3.12.4/bin"
  exit 0
fi
exit 0
UVEOF
chmod +x "$FAKE_TOOLS/uv"
run_stage 12-python-runtime-fetch.sh 1 --version 3.12.4
log_has 'uv Python installation has no executable'

# --help prints usage and exits.
build_fake_path
run_stage 12-python-runtime-fetch.sh 0 --help
log_has 'Usage:'

# ===========================================================================
# 13-tools-fetch.sh
# ===========================================================================

printf 'keycloak-theme-generator_version: "1.2.3"\n' > "$ROOT/.versions.yml"

# Missing version in .versions.yml -> WARN and skip.
build_fake_path
printf '# no version set\n' > "$ROOT/.versions.yml"
run_stage 13-tools-fetch.sh 0
log_has 'WARN: keycloak-theme-generator_version not set'

# Cached path: existing executable + matching marker -> skip.
build_fake_path
printf 'keycloak-theme-generator_version: "1.2.3"\n' > "$ROOT/.versions.yml"
mkdir -p "$ROOT/build/tools"
printf '#!/usr/bin/env bash\nexit 0\n' > "$ROOT/build/tools/keycloak-theme-generator"
chmod +x "$ROOT/build/tools/keycloak-theme-generator"
printf '1.2.3' > "$ROOT/build/tools/keycloak-theme-generator.version"
run_stage 13-tools-fetch.sh 0
log_has 'already cached. Skip.'

# No token -> abort.
build_fake_path
rm -f "$ROOT/build/tools/keycloak-theme-generator"
run_stage 13-tools-fetch.sh 1
log_has 'No GitLab token found'

# Download path with GITLAB_TOKEN: curl + unzip stubs produce the binary.
build_fake_path
rm -f "$ROOT/build/tools/keycloak-theme-generator"
cat > "$FAKE_TOOLS/curl" <<'CURLEOF'
#!/usr/bin/env bash
# curl -fLs [--header ...] <url> -o <file>
if [[ "$1" == "-fLs" ]]; then
  shift
  while [[ $# -gt 0 ]]; do
    if [[ "$1" == "-o" ]]; then
      printf 'zip-content\n' > "$2"
      exit 0
    fi
    shift
  done
fi
exit 0
CURLEOF
chmod +x "$FAKE_TOOLS/curl"
cat > "$FAKE_TOOLS/unzip" <<'UNZIPEOF'
#!/usr/bin/env bash
# unzip -p <file> <member>  -> write the "release binary" to stdout
if [[ "$1" == "-p" ]]; then
  printf '#!/usr/bin/env bash\nexit 0\n'
  exit 0
fi
exit 0
UNZIPEOF
chmod +x "$FAKE_TOOLS/unzip"
GITLAB_TOKEN=fake run_stage 13-tools-fetch.sh 0
log_has 'Using GITLAB_TOKEN for authentication'
log_has 'keycloak-theme-generator installed at'
[[ -x "$ROOT/build/tools/keycloak-theme-generator" ]] || { echo "tool must be installed." >&2; exit 1; }
[[ "$(cat "$ROOT/build/tools/keycloak-theme-generator.version")" == "1.2.3" ]] || { echo "marker mismatch." >&2; exit 1; }

# CI_JOB_TOKEN path.
build_fake_path
rm -f "$ROOT/build/tools/keycloak-theme-generator"
make_stub curl
make_stub unzip
CI_JOB_TOKEN=fake run_stage 13-tools-fetch.sh 0
log_has 'Using CI_JOB_TOKEN for authentication'

# Version mismatch triggers re-download.
build_fake_path
printf '1.2.3\n' > "$ROOT/build/tools/keycloak-theme-generator.version"
printf '#!/usr/bin/env bash\nexit 0\n' > "$ROOT/build/tools/keycloak-theme-generator"
chmod +x "$ROOT/build/tools/keycloak-theme-generator"
printf 'old\n' > "$ROOT/build/tools/keycloak-theme-generator.version"
cat > "$FAKE_TOOLS/curl" <<'CURLEOF'
#!/usr/bin/env bash
if [[ "$1" == "-fLs" ]]; then
  shift
  while [[ $# -gt 0 ]]; do
    if [[ "$1" == "-o" ]]; then
      printf 'zip-content\n' > "$2"
      exit 0
    fi
    shift
  done
fi
exit 0
CURLEOF
chmod +x "$FAKE_TOOLS/curl"
cat > "$FAKE_TOOLS/unzip" <<'UNZIPEOF'
#!/usr/bin/env bash
if [[ "$1" == "-p" ]]; then
  printf '#!/usr/bin/env bash\nexit 0\n'
  exit 0
fi
exit 0
UNZIPEOF
chmod +x "$FAKE_TOOLS/unzip"
printf 'keycloak-theme-generator_version: "1.2.3"\n' > "$ROOT/.versions.yml"
GITLAB_TOKEN=fake run_stage 13-tools-fetch.sh 0
log_has 'Version mismatch (cached: old, wanted: 1.2.3). Re-downloading...'

# ===========================================================================
# 14-runtime-compatibility-check.sh
# ===========================================================================

# Missing stage dir aborts.
build_fake_path
run_stage 14-runtime-compatibility-check.sh 1
log_has 'Missing staging directory'

make_compat_tools() {
  cat > "$FAKE_TOOLS/file" <<'FILEEOF'
#!/usr/bin/env bash
# file -b <candidate> ; classify by filename
for a in "$@"; do
  [[ "$a" == "-b" ]] && continue
  case "$a" in
    *python*) echo "ASCII text" ;;
    *aarch64*) echo "ELF 64-bit LSB pie executable, ARM aarch64" ;;
    *musl*) echo "ELF 64-bit LSB pie executable, x86-64, dynamically linked" ;;
    *notelf*) echo "ASCII text" ;;
    *) echo "ELF 64-bit LSB pie executable, x86-64, dynamically linked" ;;
  esac
  exit 0
done
exit 0
FILEEOF
  chmod +x "$FAKE_TOOLS/file"
  cat > "$FAKE_TOOLS/readelf" <<'READELFEOF'
#!/usr/bin/env bash
for a in "$@"; do
  case "$a" in
    *musl*) echo "  [Requesting program interpreter: /lib/ld-musl-x86_64.so.1]" ;;
  esac
done
exit 0
READELFEOF
  chmod +x "$FAKE_TOOLS/readelf"
  cat > "$FAKE_TOOLS/objdump" <<'OBJDUMPEOF'
#!/usr/bin/env bash
for a in "$@"; do
  case "$a" in
    *glibc234*) echo "  NEEDED  GLIBC_2.34" ;;
    *glibc238*) echo "  NEEDED  GLIBC_2.38" ;;
    *) echo "  NEEDED  GLIBC_2.28" ;;
  esac
done
exit 0
OBJDUMPEOF
  chmod +x "$FAKE_TOOLS/objdump"
}

# Missing tools abort.
build_fake_path
mkdir -p "$ROOT/stage"
make_compat_tools
rm -f "$FAKE_TOOLS/file"
run_stage 14-runtime-compatibility-check.sh 1 "$ROOT/stage"
log_has "'file' is required"

build_fake_path
mkdir -p "$ROOT/stage"
make_compat_tools
rm -f "$FAKE_TOOLS/objdump"
run_stage 14-runtime-compatibility-check.sh 1 "$ROOT/stage"
log_has "'objdump' is required"

build_fake_path
mkdir -p "$ROOT/stage"
make_compat_tools
rm -f "$FAKE_TOOLS/readelf"
run_stage 14-runtime-compatibility-check.sh 1 "$ROOT/stage"
log_has "'readelf' is required"

# Happy path: x86-64 ELF within glibc limit + embedded python imports ok.
build_fake_path
make_compat_tools
mkdir -p "$ROOT/stage/opt/opentalk-compose/runtime/python/bin"
printf '#!/usr/bin/env bash\nexit 0\n' > "$ROOT/stage/opt/opentalk-compose/runtime/python/bin/python3"
chmod +x "$ROOT/stage/opt/opentalk-compose/runtime/python/bin/python3"
printf 'x' > "$ROOT/stage/elf-bin"
chmod +x "$ROOT/stage/elf-bin"
run_stage 14-runtime-compatibility-check.sh 0 "$ROOT/stage"
log_has 'ELF files checked: 1'
log_has 'Runtime compatibility check passed.'

# Non-ELF files are skipped.
printf 'plain text\n' > "$ROOT/stage/notelf"
run_stage 14-runtime-compatibility-check.sh 0 "$ROOT/stage"
log_has 'ELF files checked: 1'

# Unsupported architecture aborts.
printf 'x' > "$ROOT/stage/aarch64-bin"
chmod +x "$ROOT/stage/aarch64-bin"
run_stage 14-runtime-compatibility-check.sh 1 "$ROOT/stage"
log_has 'Unsupported ELF architecture'

# musl ELF aborts.
rm -f "$ROOT/stage/aarch64-bin"
printf 'x' > "$ROOT/stage/musl-bin"
chmod +x "$ROOT/stage/musl-bin"
run_stage 14-runtime-compatibility-check.sh 1 "$ROOT/stage"
log_has 'musl ELF is not supported'

# glibc above limit aborts.
rm -f "$ROOT/stage/musl-bin"
printf 'x' > "$ROOT/stage/glibc238-bin"
chmod +x "$ROOT/stage/glibc238-bin"
run_stage 14-runtime-compatibility-check.sh 1 "$ROOT/stage"
log_has 'requires glibc 2.38'

# Missing embedded python aborts.
rm -f "$ROOT/stage/glibc238-bin"
rm -rf "$ROOT/stage/opt"
run_stage 14-runtime-compatibility-check.sh 1 "$ROOT/stage"
log_has 'Embedded Python missing'

echo "Stage scripts 10-14 tests passed."