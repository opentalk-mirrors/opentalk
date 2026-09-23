#!/usr/bin/env bash
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

set -euo pipefail

# Sign RPM packages with GPG
# Only runs on success and for RPM flavor

if [ "$CI_JOB_STATUS" != "success" ]; then
    echo "Job failed; skipping RPM signing."
    exit 0
fi

if [ "${FLAVOR:-}" != "rpm" ]; then
    echo "Not RPM flavor; skipping GPG signing."
    exit 0
fi

echo "Setting up GPG environment for RPM signing..."

export GNUPGHOME="$HOME/.gnupg"
mkdir -p "$GNUPGHOME"
chmod 700 "$GNUPGHOME"

GPG_TTY=$(tty 2>/dev/null || echo /dev/tty)
export GPG_TTY

# Import GPG private key from Base64-encoded variable
echo "Importing GPG key..."
echo "$GPG_PRIVATE_KEY_BASE64" | base64 --decode | gpg --batch --import

# Get the key ID from the imported key
GPG_KEY_ID=$(gpg --list-secret-keys --with-colons | awk -F: '/^sec/ {print $5; exit}')
echo "Using GPG_KEY_ID=$GPG_KEY_ID"

# Setup RPM signing configuration
echo "$GPG_PASSPHRASE" > /tmp/.gpg-passphrase
chmod 600 /tmp/.gpg-passphrase

{
    echo "%_signature gpg"
    echo "%_gpg_path $GNUPGHOME"
    echo "%_gpg_name $GPG_KEY_ID"
    echo "%__gpg_sign_cmd %{__gpg} gpg --batch --pinentry-mode loopback --passphrase-file /tmp/.gpg-passphrase --no-verbose --no-armor --no-secmem-warning -u \"%{_gpg_name}\" -sbo %{__signature_filename} %{__plaintext_filename}"
} > ~/.rpmmacros

# Sign all RPM packages
for pkg in artifacts/*.rpm; do
    if [ ! -f "$pkg" ]; then
        echo "No RPM packages found in artifacts/"
        exit 0
    fi
    echo "Signing $pkg"
    rpmsign --addsign "$pkg"
done

# Cleanup
rm -f /tmp/.gpg-passphrase

echo "RPM signing complete."
