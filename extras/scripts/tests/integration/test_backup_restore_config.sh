#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

if [[ ${OPENTALK_E2E:-} != 1 ]]; then
  echo "Set OPENTALK_E2E=1 to run the backup and restore lifecycle test." >&2
  exit 2
fi

REPO_ROOT=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." && pwd)
DEV_DIR=${OPENTALK_DEV_DIR:-"$REPO_ROOT/build/opentalk-compose-dev"}
SOURCE_DIR=$(mktemp -d)
RESTORE_DIR=$(mktemp -d)
BACKUP_DIR=$(mktemp -d)
trap 'rm -rf "$SOURCE_DIR" "$RESTORE_DIR" "$BACKUP_DIR"' EXIT

cp -a "$DEV_DIR/config/." "$SOURCE_DIR/"

just --justfile "$REPO_ROOT/justfile" _dev-ansible "--tags config_backup -e ansible_become=false -e config_dir=$SOURCE_DIR -e backup_dir=$BACKUP_DIR"

archive=$(find "$BACKUP_DIR/config" -maxdepth 1 -type f -name '*.tar.gz' -print -quit)
if [[ -z $archive ]]; then
  echo "Configuration backup archive was not created." >&2
  exit 1
fi

tar -tzf "$archive" | grep -q 'settings.yml'

just --justfile "$REPO_ROOT/justfile" _dev-ansible "--tags config_restore -e ansible_become=false -e config_dir=$RESTORE_DIR -e backup_dir=$BACKUP_DIR -e config_backup_restore_file=$(basename "$archive") -e config_restore_owner=$(id -un) -e config_restore_group=$(id -gn)"

cmp "$SOURCE_DIR/settings.yml" "$RESTORE_DIR/settings.yml"
echo "Configuration backup and restore lifecycle test passed."