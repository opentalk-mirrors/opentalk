#!/bin/bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

mkdir -p /var/opt/opentalk-compose/data
mkdir -p /var/opt/opentalk-compose/backups
chmod 755 /var/opt/opentalk-compose/data
chmod 755 /var/opt/opentalk-compose/backups
chown root:root /var/opt/opentalk-compose/data
chown root:root /var/opt/opentalk-compose/backups

MAN_FILE=/usr/share/man/man1/opentalk-compose.1
CFORGE_CMD=/usr/bin/cforge
BASH_COMPLETION_CFORGE=/etc/bash_completion.d/cforge.bash

if [ ! -f "$SETTINGS_FILE" ]; then
  cp "$ANSIBLE_DEFAULT_FILE" "$SETTINGS_FILE"
fi

if [ ! -f "$MAN_FILE" ]; then
  ln -s /opt/opentalk-compose/extras/packaging/man/opentalk-compose.1 ${MAN_FILE}
fi

if [ ! -f "$CFORGE_CMD" ]; then
  ln -s /opt/opentalk-compose/extras/packaging/cforge ${CFORGE_CMD}
fi

if [ ! -f "$BASH_COMPLETION_CFORGE" ]; then
  ln -s /opt/opentalk-compose/extras/packaging/cforge_completion.bash ${BASH_COMPLETION_CFORGE}
fi

if [ -f "$BASH_COMPLETION_CFORGE" ]; then
  # shellcheck source=/dev/null
  # Quote path (SC2086)
  source "$BASH_COMPLETION_CFORGE"
  complete -F _cforge_completion cforge
fi