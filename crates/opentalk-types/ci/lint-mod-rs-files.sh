#!/usr/bin/env bash
#
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

set -e

SELF_CMD="$0"
FD_CMD=

# Because we can't call bash functions with `fd --exec`, we choose a workaround
# that calls this very same shell script with the file path passed in as a
# parameter. Therefore if a parameter was passed, we check the file and exit
# with the appropriate exit code.
if [ $# -ne 0 ]; then
  MOD_DIRNAME="$1"
  FILENAME="${MOD_DIRNAME}.rs"

  NORMAL="\033[0m"
  NORMAL_BOLD="\033[1m"
  BLUE_BOLD="\033[34;1m"
  BLUE_NORMAL="\033[34m"

  if [ -d "$MOD_DIRNAME" ]
  then
    echo -e "File ${NORMAL_BOLD}${FILENAME}${NORMAL} should be ${BLUE_NORMAL}${MOD_DIRNAME}/${BLUE_BOLD}mod.rs${NORMAL}" >&2
    exit 1
  fi

  exit 0
fi


if FD_CMD=$(command -v fdfind || command -v fd); then
  "$FD_CMD" '\.rs$' --exec "$SELF_CMD" "{.}"
else
  echo "No \`fd\` or \`fdfind\` command found" >&2
  exit 255
fi
