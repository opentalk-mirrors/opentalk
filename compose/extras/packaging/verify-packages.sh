#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <package.deb|package.rpm> [...]" >&2
  exit 2
fi

PACKAGES=("$@")

command -v dpkg-deb >/dev/null 2>&1 || { echo "dpkg-deb is required." >&2; exit 1; }
command -v rpm >/dev/null 2>&1 || { echo "rpm is required." >&2; exit 1; }
command -v rpm2cpio >/dev/null 2>&1 || { echo "rpm2cpio is required." >&2; exit 1; }
command -v cpio >/dev/null 2>&1 || { echo "cpio is required." >&2; exit 1; }

verify_tree() {
  local tree="$1"
  local python_bin="$tree/opt/opentalk-compose/runtime/python/bin/python3"
  local ansible_bin="$tree/opt/opentalk-compose/runtime/venv/bin/ansible"

  [[ -x "$python_bin" ]] || { echo "Embedded Python missing in $tree" >&2; return 1; }
  [[ -x "$ansible_bin" ]] || { echo "Embedded Ansible missing in $tree" >&2; return 1; }
  "$python_bin" -c 'import ssl, hashlib, sqlite3, ctypes'
  "$ansible_bin" --version >/dev/null
  find "$tree" -type f -print0 | while IFS= read -r -d '' file; do
    if [[ "$file" == *extras/packaging* ]]; then
      echo "Packaging source leaked into package: $file" >&2
      return 1
    fi
  done
}

for package in "${PACKAGES[@]}"; do
  package="$(realpath "$package")"
  [[ -f "$package" ]] || { echo "Missing package: $package" >&2; exit 1; }
  tree=$(mktemp -d)
  if [[ "$package" == *.deb ]]; then
    dpkg-deb --info "$package" >/dev/null
    dpkg-deb -x "$package" "$tree"
  elif [[ "$package" == *.rpm ]]; then
    rpm -qip "$package" >/dev/null
    # No pipe: cpio stops reading at the archive trailer, and rpm2cpio from
    # RPM 6 still writes after it, so under pipefail the SIGPIPE fails the run.
    rpm2cpio "$package" > "$tree.cpio"
    (cd "$tree" && cpio -idm --quiet < "$tree.cpio")
    rm -f "$tree.cpio"
  else
    echo "Unsupported package type: $package" >&2
    rm -rf "$tree"
    exit 1
  fi
  verify_tree "$tree"
  rm -rf "$tree"
done

echo "Package smoke checks passed."
