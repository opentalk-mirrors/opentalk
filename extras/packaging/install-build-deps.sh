#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

MODE=check

usage() {
  cat <<'EOF'
Install or check host dependencies for building OpenTalk Compose packages.

Usage:
  install-build-deps.sh --check
  install-build-deps.sh --install

The default mode is --check. --install requires root privileges or sudo.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check) MODE=check; shift ;;
    --install) MODE=install; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ "$MODE" == install && $EUID -ne 0 ]]; then
  command -v sudo >/dev/null 2>&1 || {
    echo "[deps] ERROR: --install requires root privileges or sudo." >&2
    exit 1
  }
  SUDO=(sudo)
else
  SUDO=()
fi

APT_PACKAGES=(
  bash curl git jq rsync tar unzip coreutils python3
  python3-pip python3-venv python3-pip-tools ansible
  dpkg-dev rpm ruby-full just
)
DNF_PACKAGES=(
  bash curl git jq rsync tar unzip coreutils python3
  python3-pip python3-virtualenv ansible-core rpm-build dpkg ruby just
)
ZYPPER_PACKAGES=(
  bash curl git jq rsync tar unzip coreutils python3
  python3-pip python3-virtualenv ansible-core rpm-build dpkg ruby just
)
PACMAN_PACKAGES=(
  bash curl git jq rsync tar unzip coreutils python
  python-pip python-virtualenv ansible-core rpm-tools dpkg ruby just
)

missing=()
check_command() {
  local command_name="$1"
  local package_name="$2"
  if ! command -v "$command_name" >/dev/null 2>&1; then
    missing+=("$package_name")
  fi
}

check_dependencies() {
  check_command bash bash
  check_command curl curl
  check_command git git
  check_command jq jq
  check_command rsync rsync
  check_command tar tar
  check_command unzip unzip
  check_command sha256sum coreutils
  check_command python3 python3
  check_command pip3 python3-pip
  check_command ansible-galaxy ansible
  check_command uv uv
  check_command just just
  check_command gem ruby
  check_command file file
  check_command objdump binutils
  check_command readelf binutils
  if [[ "$TARGET" == rpm || "$TARGET" == both ]]; then
    check_command rpm rpm
    check_command rpm2cpio rpm
    check_command cpio cpio
  fi

  if [[ "$TARGET" == deb || "$TARGET" == both ]]; then
    if ! command -v dpkg-deb >/dev/null 2>&1; then
      missing+=("dpkg-deb")
    fi
  fi
  if [[ "$TARGET" == rpm || "$TARGET" == both ]]; then
    if ! command -v fpm >/dev/null 2>&1 && ! command -v rpmbuild >/dev/null 2>&1; then
      missing+=("fpm or rpmbuild")
    fi
  fi
}

TARGET=both
if [[ "${BUILD_TARGET:-}" == deb || "${BUILD_TARGET:-}" == rpm ]]; then
  TARGET="$BUILD_TARGET"
fi

if [[ "$MODE" == install ]]; then
  if command -v apt-get >/dev/null 2>&1; then
    "${SUDO[@]}" apt-get update
    "${SUDO[@]}" apt-get install -y "${APT_PACKAGES[@]}"
  elif command -v dnf >/dev/null 2>&1; then
    "${SUDO[@]}" dnf install -y "${DNF_PACKAGES[@]}"
  elif command -v zypper >/dev/null 2>&1; then
    "${SUDO[@]}" zypper --non-interactive refresh
    "${SUDO[@]}" zypper --non-interactive install --no-recommends "${ZYPPER_PACKAGES[@]}"
  elif command -v pacman >/dev/null 2>&1; then
    "${SUDO[@]}" pacman -Sy --needed --noconfirm "${PACMAN_PACKAGES[@]}"
  else
    echo "[deps] ERROR: unsupported distribution (need apt-get, dnf, zypper, or pacman); install dependencies manually." >&2
    exit 1
  fi
fi

check_dependencies

if (( ${#missing[@]} > 0 )); then
  printf '[deps] Missing: %s\n' "${missing[*]}" >&2
  if [[ "$MODE" == check ]]; then
    echo "[deps] Run with --install or install the missing tools manually." >&2
  fi
  exit 1
fi

echo "[deps] All required build dependencies are available."
