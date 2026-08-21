#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Stage scripts 20-50 (build chain): ansible-wheelhouse, collections-download,
# collections-vendor, venv-build, embed-assemble. Each reads build inputs from
# $PACKAGING_ROOT, which the test points at a temp "repo" it builds itself.
# External tools (pip, uv, ansible-galaxy, portable python) are provided as
# deterministic stubs in a strict PATH.

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

make_stub() {
  printf '#!/usr/bin/env bash\nexit 0\n' > "$FAKE_TOOLS/$1"
  chmod +x "$FAKE_TOOLS/$1"
}

build_fake_path() {
  # Clean tool overrides AND stale build state from previous test cases.
  rm -rf "$FAKE_TOOLS"
  mkdir -p "$FAKE_TOOLS"
  rm -rf "$ROOT/build" "$ROOT/automation" "$ROOT/extras" "$ROOT/opt"
  mkdir -p "$ROOT/build" "$ROOT/extras/packaging"
  for tool in bash env dirname pwd echo cat rm mkdir cp mv chmod ln find \
      sort tail printf awk grep sed tr sha256sum install mktemp date head wc ls; do
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

# A pip stub that writes a wheel matching the first requirement in the
# requirements file (kept in the exported PACKAGING_ROOT).
make_pip_stub() {
  cat > "$FAKE_TOOLS/pip" <<'PIPEOF'
#!/usr/bin/env bash
dest=""
prev=""
for a in "$@"; do
  [[ "$prev" == "--dest" ]] && dest="$a"
  prev="$a"
done
if [[ "$1" == "download" && -n "$dest" ]]; then
  reqfile="${PACKAGING_ROOT}/extras/packaging/requirements-ansible.txt"
  reqline=""
  while IFS= read -r l; do
    [[ "$l" =~ ^[[:space:]]*# ]] && continue
    [[ -z "${l//[[:space:]]/}" ]] && continue
    reqline="$l"
    break
  done < "$reqfile"
  pkg="${reqline%%\\*}"
  name="${pkg%%==*}"; ver="${pkg##*==}"
  : > "$dest/${name//-/_}-${ver}-py3-none-any.whl"
fi
exit 0
PIPEOF
  chmod +x "$FAKE_TOOLS/pip"
}

# A portable python stub that emulates `python -m pip download`.
make_python_dl_stub() {
  mkdir -p "$ROOT/build/python/bin"
  cat > "$ROOT/build/python/bin/python3" <<'PYEOF'
#!/usr/bin/env bash
dest=""
prev=""
for a in "$@"; do
  [[ "$prev" == "--dest" ]] && dest="$a"
  prev="$a"
done
if [[ "$1" == "-m" && "$2" == "pip" && "$3" == "download" && -n "$dest" ]]; then
  reqfile="${PACKAGING_ROOT}/extras/packaging/requirements-ansible.txt"
  reqline=""
  while IFS= read -r l; do
    [[ "$l" =~ ^[[:space:]]*# ]] && continue
    [[ -z "${l//[[:space:]]/}" ]] && continue
    reqline="$l"
    break
  done < "$reqfile"
  pkg="${reqline%%\\*}"
  name="${pkg%%==*}"; ver="${pkg##*==}"
  : > "$dest/${name//-/_}-${ver}-py3-none-any.whl"
fi
exit 0
PYEOF
  chmod +x "$ROOT/build/python/bin/python3"
}

# Write the shared requirements-ansible.txt used by scripts 20 and 40.
write_reqs() {
  printf 'ansible-core==2.16.0\n' > "$ROOT/extras/packaging/requirements-ansible.txt"
}

# A minimal executable runtime python3 at the path script 40 expects.
make_runtime_python() {
  mkdir -p "$ROOT/build/python/bin"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$ROOT/build/python/bin/python3"
  chmod +x "$ROOT/build/python/bin/python3"
}

# ===========================================================================
# 20-ansible-wheelhouse.sh
# ===========================================================================

# Missing requirements file aborts.
build_fake_path
rm -f "$ROOT/extras/packaging/requirements-ansible.txt"
run_stage 20-ansible-wheelhouse.sh 1
log_has 'Missing'

# All wheels present -> skip download.
build_fake_path
write_reqs
mkdir -p "$ROOT/build/wheelhouse"
touch "$ROOT/build/wheelhouse/ansible_core-2.16.0-py3-none-any.whl"
run_stage 20-ansible-wheelhouse.sh 0
log_has 'All wheels present; skip download.'

# Download path using the portable python.
build_fake_path
rm -rf "$ROOT/build/wheelhouse"
write_reqs
mkdir -p "$ROOT/build/wheelhouse"
make_python_dl_stub
run_stage 20-ansible-wheelhouse.sh 0
log_has 'Using portable python'
log_has 'Ready at'
[[ -e "$ROOT/build/wheelhouse/ansible_core-2.16.0-py3-none-any.whl" ]] || { echo "wheel must be downloaded." >&2; exit 1; }

# System-pip fallback when portable python is missing.
build_fake_path
rm -rf "$ROOT/build/wheelhouse" "$ROOT/build/python"
write_reqs
mkdir -p "$ROOT/build/wheelhouse"
make_pip_stub
run_stage 20-ansible-wheelhouse.sh 0
log_has 'Portable python missing, falling back to system pip'
log_has 'Ready at'

# Missing wheel after download aborts.
build_fake_path
rm -rf "$ROOT/build/wheelhouse"
write_reqs
mkdir -p "$ROOT/build/wheelhouse"
cat > "$FAKE_TOOLS/pip" <<'PIPEOF'
#!/usr/bin/env bash
dest=""
prev=""
for a in "$@"; do
  [[ "$prev" == "--dest" ]] && dest="$a"
  prev="$a"
done
if [[ "$1" == "download" && -n "$dest" ]]; then
  : > "$dest/other-pkg-9.9.9-py3-none-any.whl"
fi
exit 0
PIPEOF
chmod +x "$FAKE_TOOLS/pip"
run_stage 20-ansible-wheelhouse.sh 1
log_has 'ERROR: Missing wheel files'

# ===========================================================================
# 30-ansible-collections-download.sh
# ===========================================================================

# Missing requirements file skips (exit 0).
build_fake_path
rm -f "$ROOT/automation/collections/requirements.yml"
run_stage 30-ansible-collections-download.sh 0
log_has 'requirements file missing'

# Up-to-date: matching state hash + existing tarballs -> skip.
build_fake_path
mkdir -p "$ROOT/automation/collections" "$ROOT/build/collections_dist"
printf 'collections:\n  - name: community.general\n' > "$ROOT/automation/collections/requirements.yml"
CUR="$(sha256sum "$ROOT/automation/collections/requirements.yml" | awk '{print $1}')"
mkdir -p "$ROOT/build/.cache"
printf '%s' "$CUR" > "$ROOT/build/.cache/collections.requirements.sha256"
touch "$ROOT/build/collections_dist/community.general-1.0.0.tar.gz"
run_stage 30-ansible-collections-download.sh 0
log_has 'Up-to-date; skip download.'

# Download path: stub ansible-galaxy to emit a tarball.
build_fake_path
rm -rf "$ROOT/build/.cache" "$ROOT/build/collections_dist"
mkdir -p "$ROOT/automation/collections"
printf 'collections:\n  - name: community.general\n' > "$ROOT/automation/collections/requirements.yml"
cat > "$FAKE_TOOLS/ansible-galaxy" <<'AGXEOF'
#!/usr/bin/env bash
p=""
prev=""
for a in "$@"; do
  [[ "$prev" == "-p" ]] && p="$a"
  prev="$a"
done
if [[ "$1" == "collection" && "$2" == "download" && -n "$p" ]]; then
  mkdir -p "$p"
  : > "$p/community.general-1.0.0.tar.gz"
fi
exit 0
AGXEOF
chmod +x "$FAKE_TOOLS/ansible-galaxy"
run_stage 30-ansible-collections-download.sh 0
log_has 'Downloading (changed or forced)...'
log_has 'Tarballs at'

# ===========================================================================
# 31-ansible-collections-vendor.sh
# ===========================================================================

# Missing requirements file skips (exit 0).
build_fake_path
rm -f "$ROOT/automation/collections/requirements.yml"
run_stage 31-ansible-collections-vendor.sh 0
log_has 'requirements file missing'

# Up-to-date: vendor dir + matching state hash -> skip.
build_fake_path
mkdir -p "$ROOT/automation/collections" "$ROOT/build/vendor/collections/ansible_collections"
printf 'collections:\n  - name: community.general\n' > "$ROOT/automation/collections/requirements.yml"
CUR="$(sha256sum "$ROOT/automation/collections/requirements.yml" | awk '{print $1}')"
mkdir -p "$ROOT/build/.cache"
printf '%s' "$CUR" > "$ROOT/build/.cache/collections.vendor.sha256"
run_stage 31-ansible-collections-vendor.sh 0
log_has 'Up-to-date; skip.'

# dist dir missing -> abort.
build_fake_path
mkdir -p "$ROOT/automation/collections"
printf 'collections:\n  - name: community.general\n' > "$ROOT/automation/collections/requirements.yml"
run_stage 31-ansible-collections-vendor.sh 1
log_has 'dist dir missing'

# No tarballs in dist -> abort.
build_fake_path
mkdir -p "$ROOT/automation/collections" "$ROOT/build/collections_dist"
printf 'collections:\n  - name: community.general\n' > "$ROOT/automation/collections/requirements.yml"
run_stage 31-ansible-collections-vendor.sh 1
log_has 'no tarballs in'

# Success: stub ansible-galaxy install creates both required collections.
build_fake_path
mkdir -p "$ROOT/automation/collections" "$ROOT/build/collections_dist"
printf 'collections:\n  - name: community.general\n' > "$ROOT/automation/collections/requirements.yml"
touch "$ROOT/build/collections_dist/community.general-1.0.0.tar.gz"
cat > "$FAKE_TOOLS/ansible-galaxy" <<'AGXEOF'
#!/usr/bin/env bash
p=""
prev=""
for a in "$@"; do
  [[ "$prev" == "-p" ]] && p="$a"
  prev="$a"
done
if [[ "$1" == "collection" && "$2" == "install" && -n "$p" ]]; then
  mkdir -p "$p/ansible_collections/community/general" "$p/ansible_collections/community/docker"
fi
exit 0
AGXEOF
chmod +x "$FAKE_TOOLS/ansible-galaxy"
run_stage 31-ansible-collections-vendor.sh 0
log_has 'Verified required collections'

# ansible_collections dir not created -> abort.
build_fake_path
mkdir -p "$ROOT/automation/collections" "$ROOT/build/collections_dist"
printf 'collections:\n  - name: community.general\n' > "$ROOT/automation/collections/requirements.yml"
touch "$ROOT/build/collections_dist/community.general-1.0.0.tar.gz"
make_stub ansible-galaxy
run_stage 31-ansible-collections-vendor.sh 1
log_has 'ansible_collections directory not created'

# Missing required collection (only one present) -> abort.
build_fake_path
mkdir -p "$ROOT/automation/collections" "$ROOT/build/collections_dist"
printf 'collections:\n  - name: community.general\n' > "$ROOT/automation/collections/requirements.yml"
touch "$ROOT/build/collections_dist/community.general-1.0.0.tar.gz"
cat > "$FAKE_TOOLS/ansible-galaxy" <<'AGXEOF'
#!/usr/bin/env bash
p=""
prev=""
for a in "$@"; do
  [[ "$prev" == "-p" ]] && p="$a"
  prev="$a"
done
if [[ "$1" == "collection" && "$2" == "install" && -n "$p" ]]; then
  mkdir -p "$p/ansible_collections/community/general"
fi
exit 0
AGXEOF
chmod +x "$FAKE_TOOLS/ansible-galaxy"
run_stage 31-ansible-collections-vendor.sh 2
log_has "required collection 'community.docker' missing"

# ===========================================================================
# 40-ansible-venv-build.sh
# ===========================================================================

# uv missing -> abort.
build_fake_path
rm -f "$FAKE_TOOLS/uv"
run_stage 40-ansible-venv-build.sh 1
log_has 'uv is required'

# Wheelhouse empty -> abort.
build_fake_path
make_stub uv
write_reqs
run_stage 40-ansible-venv-build.sh 1
log_has 'Wheelhouse empty'

# Missing requirements file -> abort.
build_fake_path
make_stub uv
write_reqs
rm -f "$ROOT/extras/packaging/requirements-ansible.txt"
mkdir -p "$ROOT/build/wheelhouse"
touch "$ROOT/build/wheelhouse/ansible_core-2.16.0-py3-none-any.whl"
run_stage 40-ansible-venv-build.sh 1
log_has 'Missing'

# No python runtime -> abort.
build_fake_path
make_stub uv
write_reqs
mkdir -p "$ROOT/build/wheelhouse"
touch "$ROOT/build/wheelhouse/ansible_core-2.16.0-py3-none-any.whl"
PYBIN="$ROOT/build/python/bin/python3" run_stage 40-ansible-venv-build.sh 1
log_has 'No python runtime.'

# Up-to-date: matching marker + ansible binary present -> skip.
build_fake_path
make_stub uv
make_runtime_python
write_reqs
mkdir -p "$ROOT/build/wheelhouse" "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv/bin"
touch "$ROOT/build/wheelhouse/ansible_core-2.16.0-py3-none-any.whl"
printf '#!/usr/bin/env bash\nexit 0\n' > "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv/bin/ansible"
chmod +x "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv/bin/ansible"
CUR="$(sha256sum "$ROOT/extras/packaging/requirements-ansible.txt" | awk '{print $1}')"
printf '%s' "$CUR" > "$ROOT/build/pkgroot/opt/opentalk-compose/runtime/venv/.reqs.sha256"
run_stage 40-ansible-venv-build.sh 0
log_has 'Up-to-date; skip rebuild.'

# Build path: stub uv to create the venv bin/ansible.
build_fake_path
make_stub uv
make_runtime_python
write_reqs
mkdir -p "$ROOT/build/wheelhouse" "$ROOT/build/pkgroot/opt/opentalk-compose/runtime"
touch "$ROOT/build/wheelhouse/ansible_core-2.16.0-py3-none-any.whl"
cat > "$FAKE_TOOLS/uv" <<'UVEOF'
#!/usr/bin/env bash
if [[ "$1" == "venv" ]]; then
  d=""
  prev=""
  for a in "$@"; do
    if [[ "$prev" == "--python" ]]; then
      prev=""
      continue
    fi
    [[ "$a" != -* ]] && d="$a"
    prev="$a"
  done
  mkdir -p "$d/bin"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$d/bin/python"
  printf '#!/usr/bin/env bash\nexit 0\n' > "$d/bin/ansible"
  chmod +x "$d/bin/python" "$d/bin/ansible"
fi
exit 0
UVEOF
chmod +x "$FAKE_TOOLS/uv"
run_stage 40-ansible-venv-build.sh 0
log_has '(Re)building...'
log_has 'Ready.'

# ===========================================================================
# 50-embed-assemble.sh
# ===========================================================================

# build/vendor present -> copy from build/vendor.
build_fake_path
mkdir -p "$ROOT/build/vendor/collections/ansible_collections/community/general"
run_stage 50-embed-assemble.sh 0
log_has 'Copy vendor from build/vendor'

# Legacy build/collections/vendor present -> copy from legacy.
build_fake_path
mkdir -p "$ROOT/build/collections/vendor/collections/ansible_collections/community/general"
run_stage 50-embed-assemble.sh 0
log_has 'Copy vendor from legacy build/collections/vendor'

# No vendor collections -> warn.
build_fake_path
run_stage 50-embed-assemble.sh 0
log_has 'WARN: No vendor collections found'

echo "Stage scripts 20-50 tests passed."
