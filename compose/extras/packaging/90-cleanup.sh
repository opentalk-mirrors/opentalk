#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# shellcheck disable=SC2015  # Chained 'cd && RELOCATED=1 || { ... }' pattern intentional for concise error handling.
set -euo pipefail
#
# Cleanup build artifacts and temporary files
#
# Usage:
#   bash extras/packaging/90-cleanup.sh [options]
#
# Options:
#   --all, -a          Remove everything (build + artifacts + loose files)
#   --artifacts        Remove only artifacts directory and loose packages
#   --build-only       Remove only build directory
#   --help, -h         Show this help
#   -y                 Skip confirmation prompts
#
# Default: Remove build directory only (preserve artifacts)

ROOT="${CLEANUP_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
FORCE=0
MODE="standard"
RELOCATED=0

show_help() {
  cat << EOF
Cleanup build artifacts and temporary files

Usage: $(basename "$0") [options]

Options:
  --all, -a          Remove everything (build + artifacts + loose files)
  --artifacts        Remove only artifacts directory and loose packages
  --build-only       Remove only build directory (default)
  --help, -h         Show this help
  -y                 Skip confirmation prompts

Examples:
  $(basename "$0")              # Standard cleanup (build/ only)
  $(basename "$0") --all -y     # Remove everything without prompts
  $(basename "$0") --artifacts  # Clean artifacts but keep build cache
EOF
}

cleanup_artifacts() {
  echo "[cleanup] Removing build artifacts..."
  if [[ -d "$ROOT/artifacts" ]]; then
    rm -rf "$ROOT/artifacts"
    echo "[cleanup] - Removed artifacts/"
  fi
  
  local found=0
  for pattern in "*.deb" "*.rpm" "*.tar.gz" "*.tar.xz"; do
    mapfile -t matches < <(compgen -G "$ROOT/$pattern" 2>/dev/null)
    if (( ${#matches[@]} > 0 )); then
      rm -f "${matches[@]}"
      echo "[cleanup] - Removed $pattern files"
      found=1
    fi
  done
  
  [[ $found -eq 0 ]] && echo "[cleanup] - No loose package files found"
  echo "[cleanup] Artifacts cleanup completed"
}

cleanup_build() {
  echo "[cleanup] Removing build directory..."
  if [[ -d "$ROOT/build" ]]; then
    rm -rf "$ROOT/build"
    echo "[cleanup] - Removed build/"
  else
    echo "[cleanup] - No build directory found"
  fi
  echo "[cleanup] Build cleanup completed"
}

cleanup_all() {
  echo "[cleanup] Full cleanup (build + artifacts + loose files)..."
  cleanup_build
  cleanup_artifacts
  echo "[cleanup] Full cleanup completed"
}

confirm() {
  local message="$1"
  if [[ $FORCE -eq 1 ]]; then
    return 0
  fi
  
  echo -n "$message [y/N]: "
  read -r response
  case "$response" in
    [yY]|[yY][eE][sS]) return 0 ;;
    *) return 1 ;;
  esac
}

while [[ $# -gt 0 ]]; do
  case $1 in
    --all|-a) MODE="all"; shift ;;
    --artifacts) MODE="artifacts"; shift ;;
    --build-only) MODE="build"; shift ;;
    -y) FORCE=1; shift ;;
    --help|-h) show_help; exit 0 ;;
    *) echo "Unknown option: $1" >&2; echo "Use --help for usage information" >&2; exit 1 ;;
  esac
done

case $MODE in
  all)
    # Guard: if inside directory being removed, relocate to ROOT first
    if [[ "$PWD" == "$ROOT/artifacts" || "$PWD" == "$ROOT/build" || "$PWD" == $ROOT/artifacts/* || "$PWD" == $ROOT/build/* ]]; then
      echo "[cleanup] INFO: current working directory is inside target (will relocate to $ROOT before deletion)."
      cd "$ROOT" && RELOCATED=1 || { echo "[cleanup] ERROR: failed to relocate from $PWD" >&2; exit 1; }
    fi
    echo "[cleanup] Will remove: build/ + artifacts/ + loose packages"
    confirm "Proceed with full cleanup?" || exit 0
    cleanup_all
    ;;
  artifacts)
    if [[ "$PWD" == "$ROOT/artifacts" || "$PWD" == $ROOT/artifacts/* ]]; then
      echo "[cleanup] INFO: current working directory is inside artifacts/ (relocating to $ROOT)."
      cd "$ROOT" && RELOCATED=1 || { echo "[cleanup] ERROR: failed to relocate from $PWD" >&2; exit 1; }
    fi
    echo "[cleanup] Will remove: artifacts/ + loose packages (preserving build/)"
    confirm "Proceed with artifacts cleanup?" || exit 0
    cleanup_artifacts
    ;;
  build)
    if [[ "$PWD" == "$ROOT/build" || "$PWD" == $ROOT/build/* ]]; then
      echo "[cleanup] INFO: current working directory is inside build/ (relocating to $ROOT)."
      cd "$ROOT" && RELOCATED=1 || { echo "[cleanup] ERROR: failed to relocate from $PWD" >&2; exit 1; }
    fi
    echo "[cleanup] Will remove: build/ (preserving artifacts/)"
    confirm "Proceed with build cleanup?" || exit 0
    cleanup_build
    ;;
  standard)
    if [[ "$PWD" == "$ROOT/build" || "$PWD" == $ROOT/build/* ]]; then
      echo "[cleanup] INFO: current working directory is inside build/ (relocating to $ROOT)."
      cd "$ROOT" && RELOCATED=1 || { echo "[cleanup] ERROR: failed to relocate from $PWD" >&2; exit 1; }
    fi
    echo "[cleanup] Standard cleanup: build/ only (preserving artifacts/)"
    confirm "Proceed with standard cleanup?" || exit 0
    cleanup_build
    ;;
esac

if [[ $RELOCATED -eq 1 ]]; then
  echo "[cleanup] NOTE: Working directory was inside removed tree; relocated to $ROOT."
fi
echo "[cleanup] Cleanup completed successfully"
