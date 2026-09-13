#!/usr/bin/env bash
#
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

codify() {
  if [ -z "$1" ]; then
    echo "Error: no language specified"
    return 1
  fi

  echo "\`\`\`$1"
  while IFS= read -r data; do
    echo "$data"
  done
  echo '```'
}
