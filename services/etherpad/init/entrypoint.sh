#!/bin/sh

# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

APIKEY_FILE=/opt/etherpad-lite/APIKEY.txt

if [ -n "${EP_APIKEY}" ]; then
	echo "${EP_APIKEY}" >"${APIKEY_FILE}"
elif [ -n "${EP_APIKEY_FILE}" ]; then
	ln -sf "${EP_APIKEY_FILE}" "${APIKEY_FILE}"
fi

if [ -f "${APIKEY_FILE}" ]; then
	pnpm run prod
else
	echo "error: apikey not defined"
	exit 1
fi
