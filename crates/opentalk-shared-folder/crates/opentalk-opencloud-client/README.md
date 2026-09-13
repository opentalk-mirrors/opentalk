# OpenTalk OpenCloud Client Library

This project is an OpenCloud client library for use inside OpenTalk. It therefore
contains the functionality required there, and does not intend to provide full
API coverage. The client is focused on creating folders (via WebDAV) and public
share links (via the LibreGraph API).

## Command-line example

The crate ships with an example CLI that exercises the client. It reads the
connection details from the `OPENCLOUD_BASE_URL`, `OPENCLOUD_USERNAME` and
`OPENCLOUD_PASSWORD` environment variables (each can also be passed as a
`--base-url`, `--username` or `--password` flag). The password may be an
[app token](https://docs.opencloud.eu/docs/next/user/admin/app-tokens/).

```sh
export OPENCLOUD_BASE_URL="https://opencloud.example.com"
export OPENCLOUD_USERNAME="alice"
export OPENCLOUD_PASSWORD="app-token-or-password"

# List the drives (spaces) available to the user, printing
# "<drive-id>\t<drive-type>\t<name>" per line
cargo run --example opentalk-opencloud-cli -- list-drives

# Create a folder inside a drive
cargo run --example opentalk-opencloud-cli -- \
  create-folder "$DRIVE_ID" "Meetings/2026"

# Inspect the share-link password policy, or generate a conforming password
cargo run --example opentalk-opencloud-cli -- password-policy
PASSWORD=$(cargo run --quiet --example opentalk-opencloud-cli -- generate-password)

# Create a public read-only share link for the folder
cargo run --example opentalk-opencloud-cli -- \
  create-link "$DRIVE_ID" "Meetings/2026" \
  --link-type view \
  --password "$PASSWORD" \
  --expiration 2026-12-31T23:59:59Z

# Update an existing link (one property per invocation)
cargo run --example opentalk-opencloud-cli -- \
  update-link "$DRIVE_ID" "$ITEM_ID" "$PERMISSION_ID" --link-type edit

# Delete a share link, then the folder
cargo run --example opentalk-opencloud-cli -- \
  delete-link "$DRIVE_ID" "$ITEM_ID" "$PERMISSION_ID"
cargo run --example opentalk-opencloud-cli -- \
  delete-folder "$DRIVE_ID" "Meetings/2026"
```

Valid `--link-type` values are `internal`, `view`, `upload`, `edit`,
`createOnly` and `blocksDownload` (`upload` and `createOnly` only apply to
folders).

Share-link passwords must conform to the server's password policy (queried from
`/ocs/v1.php/cloud/capabilities`). Use the library's `generate_password` to
obtain a conforming password, or `get_password_policy` to inspect the rules.
